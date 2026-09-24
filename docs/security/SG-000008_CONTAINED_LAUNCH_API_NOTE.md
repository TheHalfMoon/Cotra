# SG-000008 contained launch API note

Date: 2026-09-24

SG-000008 combines the stable Windows primitives qualified separately by SG-000006 and SG-000007.

The native Windows probe:
1. creates a unique temporary AppContainer profile with zero capabilities;
2. builds STARTUPINFOEX with PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES;
3. launches a fixed Windows system child with CreateProcessW using CREATE_SUSPENDED;
4. passes no inherited handles and a minimal secret-free Unicode environment;
5. creates/configures a kill-on-close Job Object;
6. assigns the suspended child to the Job Object;
7. verifies Job membership with IsProcessInJob before resuming;
8. verifies TokenIsAppContainer on the child token;
9. resumes the child;
10. waits with a finite timeout, records its exit code, and cleans up.

Security interpretation:
- AppContainer is the resource-isolation boundary.
- Job Object is lifecycle/resource control, not the sandbox by itself.
- CREATE_SUSPENDED prevents user code from running before Job assignment.
- zero AppContainer capabilities means this grain grants no network capability.
- the fixed probe does not expose arbitrary execution to MCP.
- process.spawn and PowerShell remain denied after this grain.

Official Microsoft references:
- https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer
- https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-security_capabilities
- https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-initializeprocthreadattributelist
- https://learn.microsoft.com/en-us/windows/desktop/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute
- https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
- https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject
- https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-isprocessinjob
- https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-resumethread
- https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-gettokeninformation
- https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-for-legacy-applications-

Current Windows documentation also describes Experimental_CreateProcessInSandbox. Cotra does not use it as the v1 foundation because Microsoft explicitly marks that API experimental and subject to change. It remains a future evaluation candidate.

ABI provenance:
- PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES value cross-checked against Microsoft windows-rs / Win32 metadata.
- No Microsoft implementation source code is copied.
- FFI structures and declarations mirror documented public Win32 ABI only.
