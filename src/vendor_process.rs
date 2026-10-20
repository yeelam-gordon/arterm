//! Kill-on-close ownership for short-lived vendor metadata/authentication commands.
use anyhow::{ensure, Result};
use std::{
    ffi::OsStr, fs::File, mem::{size_of, size_of_val, zeroed},
    os::windows::{ffi::OsStrExt, io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle}, process::ExitStatusExt},
    path::Path, process::ExitStatus, ptr,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE_FLAG_INHERIT, SetHandleInformation, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Security::SECURITY_ATTRIBUTES,
    System::{
        JobObjects::{CreateJobObjectW, SetInformationJobObject, JobObjectExtendedLimitInformation,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE},
        Pipes::CreatePipe,
        Threading::{CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
            InitializeProcThreadAttributeList, UpdateProcThreadAttribute, WaitForSingleObject,
            CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT,
            LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
            PROC_THREAD_ATTRIBUTE_JOB_LIST, STARTF_USESTDHANDLES, STARTUPINFOEXW},
    },
};

struct Attributes(Vec<usize>);
impl Attributes {
    fn new() -> Result<Self> {
        let mut bytes = 0;
        unsafe { InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut bytes); }
        ensure!(bytes > 0, "cannot size vendor startup attributes");
        let mut storage = vec![0; bytes.div_ceil(size_of::<usize>())];
        ensure!(unsafe { InitializeProcThreadAttributeList(storage.as_mut_ptr().cast(), 2, 0, &mut bytes) } != 0,
            "cannot initialize vendor startup attributes: {}", std::io::Error::last_os_error());
        Ok(Self(storage))
    }
    fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST { self.0.as_mut_ptr().cast() }
    fn set(&mut self, key: usize, values: &[RawHandle]) -> Result<()> {
        ensure!(unsafe { UpdateProcThreadAttribute(self.pointer(), 0, key,
            values.as_ptr().cast_mut().cast(), size_of_val(values), ptr::null_mut(), ptr::null_mut()) } != 0,
            "cannot set vendor startup attribute: {}", std::io::Error::last_os_error());
        Ok(())
    }
}
impl Drop for Attributes {
    fn drop(&mut self) { unsafe { DeleteProcThreadAttributeList(self.pointer()); } }
}

fn pipe(parent_reads: bool) -> Result<(OwnedHandle, OwnedHandle)> {
    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32, lpSecurityDescriptor: ptr::null_mut(), bInheritHandle: 1,
    };
    let (mut read, mut write) = (ptr::null_mut(), ptr::null_mut());
    ensure!(unsafe { CreatePipe(&mut read, &mut write, &mut attributes, 0) } != 0,
        "cannot create vendor pipe: {}", std::io::Error::last_os_error());
    let (read, write) = unsafe { (OwnedHandle::from_raw_handle(read), OwnedHandle::from_raw_handle(write)) };
    let parent = if parent_reads { &read } else { &write };
    ensure!(unsafe { SetHandleInformation(parent.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } != 0,
        "cannot protect vendor pipe handle: {}", std::io::Error::last_os_error());
    Ok((read, write))
}

fn quote(argument: &OsStr) -> Result<Vec<u16>> {
    let mut encoded = vec![b'"' as u16];
    let mut slashes = 0;
    for ch in argument.encode_wide() {
        ensure!(ch != 0, "vendor argument contains NUL");
        if ch == b'\\' as u16 { slashes += 1; continue; }
        let count = if ch == b'"' as u16 { slashes * 2 + 1 } else { slashes };
        encoded.extend(std::iter::repeat_n(b'\\' as u16, count));
        slashes = 0;
        encoded.push(ch);
    }
    encoded.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
    encoded.push(b'"' as u16);
    Ok(encoded)
}

pub(super) struct VendorProcess {
    _job: OwnedHandle,
    process: OwnedHandle,
    pub stdout: Option<File>,
    pub stderr: Option<File>,
}

impl VendorProcess {
    pub fn spawn(program: &Path, args: &[&str], hidden: bool) -> Result<Self> {
        let job = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        ensure!(!job.is_null(), "cannot create vendor job: {}", std::io::Error::last_os_error());
        let job = unsafe { OwnedHandle::from_raw_handle(job) };
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        ensure!(unsafe { SetInformationJobObject(job.as_raw_handle(), JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(), size_of_val(&limits) as u32) } != 0,
            "cannot configure vendor job: {}", std::io::Error::last_os_error());
        let (stdin_read, stdin_write) = pipe(false)?;
        let (stdout_read, stdout_write) = pipe(true)?;
        let (stderr_read, stderr_write) = pipe(true)?;
        let handles = [stdin_read.as_raw_handle(), stdout_write.as_raw_handle(), stderr_write.as_raw_handle()];
        let jobs = [job.as_raw_handle()];
        let mut attributes = Attributes::new()?;
        attributes.set(PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize, &handles)?;
        // Kernel assignment at creation closes the spawn/assign cancellation race.
        attributes.set(PROC_THREAD_ATTRIBUTE_JOB_LIST as usize, &jobs)?;
        let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput = handles[0];
        startup.StartupInfo.hStdOutput = handles[1];
        startup.StartupInfo.hStdError = handles[2];
        startup.lpAttributeList = attributes.pointer();
        let mut command = quote(program.as_os_str())?;
        for arg in args {
            command.push(b' ' as u16);
            command.extend(quote(OsStr::new(arg))?);
        }
        command.push(0);
        let mut process: PROCESS_INFORMATION = unsafe { zeroed() };
        let flags = CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT
            | if hidden { CREATE_NO_WINDOW } else { 0 };
        ensure!(unsafe { CreateProcessW(ptr::null(), command.as_mut_ptr(), ptr::null(), ptr::null(), 1,
            flags, ptr::null(), ptr::null(), &startup.StartupInfo, &mut process) } != 0,
            "cannot start owned vendor command: {}", std::io::Error::last_os_error());
        let process_handle = unsafe { OwnedHandle::from_raw_handle(process.hProcess) };
        unsafe { CloseHandle(process.hThread); }
        drop(stdin_write);
        Ok(Self { _job: job, process: process_handle,
            stdout: Some(File::from(stdout_read)), stderr: Some(File::from(stderr_read)) })
    }

    pub fn try_wait(&self) -> Result<Option<ExitStatus>> {
        match unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0;
                ensure!(unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } != 0,
                    "cannot read vendor exit status: {}", std::io::Error::last_os_error());
                Ok(Some(ExitStatus::from_raw(code)))
            }
            _ => Err(std::io::Error::last_os_error().into()),
        }
    }
}
