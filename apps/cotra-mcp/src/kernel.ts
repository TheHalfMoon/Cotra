import { randomUUID } from "node:crypto";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { createInterface } from "node:readline";

export interface KernelRequest {
  version: 1;
  request_id: string;
  client_session_id: string;
  workspace_id: string;
  capability: string;
  operation: string;
  target?: string;
  arguments: Record<string, unknown>;
}

export interface KernelResponse {
  version: number;
  request_id: string;
  ok: boolean;
  result?: unknown;
  error?: {
    code: string;
    message: string;
  };
  evidence?: {
    workspace_id: string;
    policy_revision: string;
    target?: string;
  };
}

type Pending = {
  resolve: (value: KernelResponse) => void;
  reject: (reason: Error) => void;
  timer: NodeJS.Timeout;
};

const SAFE_ENV_NAMES = new Set([
  "PATH",
  "Path",
  "PATHEXT",
  "SystemRoot",
  "WINDIR",
  "COMSPEC",
  "TEMP",
  "TMP",
  "USERPROFILE",
  "LOCALAPPDATA",
  "APPDATA",
  "PROGRAMDATA",
  "ProgramFiles",
  "ProgramFiles(x86)",
  "NUMBER_OF_PROCESSORS",
  "PROCESSOR_ARCHITECTURE"
]);

const SECRETISH = /(secret|token|password|passwd|credential|api[_-]?key|tunnel[_-]?key)/i;

export function buildDaemonEnv(
  source: NodeJS.ProcessEnv = process.env
): NodeJS.ProcessEnv {
  const output: NodeJS.ProcessEnv = {};

  for (const [name, value] of Object.entries(source)) {
    if (value === undefined) continue;
    if (SAFE_ENV_NAMES.has(name)) {
      output[name] = value;
      continue;
    }
    if (name.startsWith("COTRA_") && !SECRETISH.test(name)) {
      output[name] = value;
    }
  }

  return output;
}

export function buildRequest(input: {
  sessionId: string;
  workspaceId: string;
  capability: string;
  operation: string;
  target?: string | undefined;
  arguments?: Record<string, unknown> | undefined;
}): KernelRequest {
  const request: KernelRequest = {
    version: 1,
    request_id: randomUUID(),
    client_session_id: input.sessionId,
    workspace_id: input.workspaceId,
    capability: input.capability,
    operation: input.operation,
    arguments: input.arguments ?? {}
  };
  if (input.target !== undefined) {
    request.target = input.target;
  }
  return request;
}

export class KernelClient {
  readonly sessionId = randomUUID();
  readonly child: ChildProcessWithoutNullStreams;
  readonly pending = new Map<string, Pending>();

  constructor(command = process.env.COTRA_DAEMON ?? "cotrad") {
    this.child = spawn(command, [], {
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
      env: buildDaemonEnv()
    });

    this.child.stderr.on("data", (chunk: Buffer) => {
      process.stderr.write("[cotrad] " + chunk.toString("utf8"));
    });

    const lines = createInterface({ input: this.child.stdout });
    lines.on("line", (line) => this.handleLine(line));

    this.child.on("error", (error) => {
      this.rejectAll(new Error("cotrad failed to start: " + error.message));
    });
    this.child.on("exit", (code, signal) => {
      this.rejectAll(
        new Error(
          "cotrad exited while requests were pending (code=" +
            String(code) +
            " signal=" +
            String(signal) +
            ")"
        )
      );
    });
  }

  async call(input: {
    workspaceId: string;
    capability: string;
    operation: string;
    target?: string | undefined;
    arguments?: Record<string, unknown> | undefined;
    timeoutMs?: number | undefined;
  }): Promise<KernelResponse> {
    const request = buildRequest({
      sessionId: this.sessionId,
      workspaceId: input.workspaceId,
      capability: input.capability,
      operation: input.operation,
      target: input.target,
      arguments: input.arguments
    });

    const timeoutMs = input.timeoutMs ?? 30_000;
    const response = new Promise<KernelResponse>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(request.request_id);
        reject(new Error("cotrad request timed out after " + timeoutMs + "ms"));
      }, timeoutMs);
      this.pending.set(request.request_id, { resolve, reject, timer });
    });

    const payload = JSON.stringify(request) + "\n";
    this.child.stdin.write(payload, "utf8", (error) => {
      if (!error) return;
      const pending = this.pending.get(request.request_id);
      if (!pending) return;
      clearTimeout(pending.timer);
      this.pending.delete(request.request_id);
      pending.reject(error);
    });

    return response;
  }

  close(): void {
    this.rejectAll(new Error("kernel client closed"));
    this.child.stdin.end();
    if (!this.child.killed) {
      this.child.kill();
    }
  }

  private handleLine(line: string): void {
    let response: KernelResponse;
    try {
      response = JSON.parse(line) as KernelResponse;
    } catch {
      process.stderr.write("[cotra-mcp] ignored malformed cotrad response\n");
      return;
    }

    const pending = this.pending.get(response.request_id);
    if (!pending) return;
    clearTimeout(pending.timer);
    this.pending.delete(response.request_id);
    pending.resolve(response);
  }

  private rejectAll(error: Error): void {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
  }
}
