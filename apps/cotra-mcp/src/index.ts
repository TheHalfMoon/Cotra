import { McpServer } from "@modelcontextprotocol/server";
import { serveStdio } from "@modelcontextprotocol/server/stdio";
import * as z from "zod/v4";
import { KernelClient, type KernelResponse } from "./kernel.js";

const DEFAULT_WORKSPACE = process.env.COTRA_DEFAULT_WORKSPACE ?? "default";
const clients = new Set<KernelClient>();

function asToolResult(response: KernelResponse) {
  if (!response.ok) {
    return {
      isError: true,
      content: [
        {
          type: "text" as const,
          text: JSON.stringify(
            {
              error: response.error ?? {
                code: "INTERNAL_ERROR",
                message: "cotrad returned an unspecified failure"
              }
            },
            null,
            2
          )
        }
      ]
    };
  }

  return {
    content: [
      {
        type: "text" as const,
        text: JSON.stringify(
          {
            result: response.result,
            evidence: response.evidence
          },
          null,
          2
        )
      }
    ]
  };
}

function createServer(): McpServer {
  const kernel = new KernelClient();
  clients.add(kernel);

  const server = new McpServer({
    name: "cotra",
    version: "0.1.0"
  });

  const workspaceSchema = z.object({
    workspace_id: z.string().min(1).default(DEFAULT_WORKSPACE)
  });

  server.registerTool(
    "system_status",
    {
      description: "Read Cotra daemon status. This tool does not mutate the computer.",
      inputSchema: workspaceSchema
    },
    async ({ workspace_id }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "system.status",
          operation: "get"
        })
      )
  );

  server.registerTool(
    "workspace_get",
    {
      description: "Read the configured trusted workspace metadata.",
      inputSchema: workspaceSchema
    },
    async ({ workspace_id }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "workspace.get",
          operation: "get"
        })
      )
  );

  server.registerTool(
    "fs_stat",
    {
      description: "Read metadata for a relative path inside a trusted workspace.",
      inputSchema: z.object({
        workspace_id: z.string().min(1).default(DEFAULT_WORKSPACE),
        path: z.string().min(1)
      })
    },
    async ({ workspace_id, path }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "fs.stat",
          operation: "stat",
          target: path
        })
      )
  );

  server.registerTool(
    "fs_list",
    {
      description: "List one directory inside a trusted workspace without following symlinks.",
      inputSchema: z.object({
        workspace_id: z.string().min(1).default(DEFAULT_WORKSPACE),
        path: z.string().min(1).default(".")
      })
    },
    async ({ workspace_id, path }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "fs.list",
          operation: "list",
          target: path
        })
      )
  );

  server.registerTool(
    "fs_read",
    {
      description: "Read one bounded UTF-8 text file inside a trusted workspace.",
      inputSchema: z.object({
        workspace_id: z.string().min(1).default(DEFAULT_WORKSPACE),
        path: z.string().min(1)
      })
    },
    async ({ workspace_id, path }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "fs.read",
          operation: "read",
          target: path
        })
      )
  );

  server.registerTool(
    "fs_search",
    {
      description:
        "Search bounded UTF-8 text files inside a trusted workspace. Symlinks are not followed.",
      inputSchema: z.object({
        workspace_id: z.string().min(1).default(DEFAULT_WORKSPACE),
        path: z.string().min(1).default("."),
        query: z.string().min(1),
        max_results: z.number().int().min(1).max(200).default(50)
      })
    },
    async ({ workspace_id, path, query, max_results }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "fs.search",
          operation: "search",
          target: path,
          arguments: { query, max_results }
        })
      )
  );

  server.registerTool(
    "fs_write_preview",
    {
      description:
        "Preview an exact UTF-8 file write. This does not mutate the computer and returns the current/new SHA-256 values required for an approved write.",
      inputSchema: z.object({
        workspace_id: z.string().min(1).default(DEFAULT_WORKSPACE),
        path: z.string().min(1),
        content: z.string().max(2 * 1024 * 1024)
      })
    },
    async ({ workspace_id, path, content }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "fs.write",
          operation: "preview",
          target: path,
          arguments: { content }
        })
      )
  );

  server.registerTool(
    "fs_write",
    {
      description:
        "Write exact UTF-8 content inside a trusted workspace after an independent local Cotra approval. Existing files require the current SHA-256 from fs_write_preview.",
      inputSchema: z.object({
        workspace_id: z.string().min(1).default(DEFAULT_WORKSPACE),
        path: z.string().min(1),
        content: z.string().max(2 * 1024 * 1024),
        expected_current_sha256: z.string().regex(/^[0-9a-f]{64}$/).optional(),
        create_if_missing: z.boolean().default(false)
      })
    },
    async ({
      workspace_id,
      path,
      content,
      expected_current_sha256,
      create_if_missing
    }) =>
      asToolResult(
        await kernel.call({
          workspaceId: workspace_id,
          capability: "fs.write",
          operation: "write",
          target: path,
          arguments: {
            content,
            expected_current_sha256,
            create_if_missing
          },
          timeoutMs: 5 * 60_000
        })
      )
  );

  server.server.onclose = () => {
    kernel.close();
    clients.delete(kernel);
  };

  return server;
}

const handle = serveStdio(createServer);
process.stderr.write("[cotra-mcp] serving Cotra SG-000002 tools over stdio\n");

function shutdown(): void {
  for (const client of clients) {
    client.close();
  }
  clients.clear();
  void handle.close().finally(() => process.exit(0));
}

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);
