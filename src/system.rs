//! Access the native system.
use crate::graphics::compositor;
use crate::runtime::{task, Action, Task};

pub use crate::runtime::system::Information;

/// Query for available system information.
pub fn fetch_information() -> Task<Information> {
    task::oneshot(|channel| {
        Action::System(crate::runtime::system::Action::QueryInformation(channel))
    })
}

pub(crate) fn information(graphics_info: compositor::Information) -> Information {
    use sysinfo::{Process, System};
    let mut system = System::new_all();
    system.refresh_all();

    let cpu = system.global_cpu_info();

    let memory_used = sysinfo::get_current_pid()
        .and_then(|pid| system.process(pid).ok_or("Process not found"))
        .map(Process::memory)
        .ok();

    Information {
        system_name: System::name(),
        system_kernel: System::kernel_version(),
        system_version: System::long_os_version(),
        system_short_version: System::os_version(),
        cpu_brand: cpu.brand().into(),
        cpu_cores: system.physical_core_count(),
        memory_total: system.total_memory(),
        memory_used,
        graphics_adapter: graphics_info.adapter,
        graphics_backend: graphics_info.backend,
    }
}
