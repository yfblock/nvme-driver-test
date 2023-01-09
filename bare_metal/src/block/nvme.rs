use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, Ordering};
use nvme_driver::{DmaAllocator, IrqController, NvmeInterface};

use lazy_static::lazy_static;

lazy_static! {
    static ref DMA_PADDR: AtomicUsize = AtomicUsize::new(0x80700000 as usize);
}

pub struct DmaAllocatorImpl;
impl DmaAllocator for DmaAllocatorImpl {
    fn dma_alloc(size: usize) -> usize{
        let paddr = DMA_PADDR.fetch_add(size, Ordering::SeqCst);
        paddr
    }

    fn dma_dealloc(_addr: usize, _size: usize) -> usize{
        0
    }

    fn phys_to_virt(phys: usize) -> usize{
        phys
    }

    fn virt_to_phys(virt: usize) -> usize{
        virt
    }
}

pub struct IrqControllerImpl;

impl IrqController for IrqControllerImpl {
    fn enable_irq(_irq: usize){

    }

    fn disable_irq(_irq: usize){

    }
}


// 虚拟IO设备
pub struct VirtIOBlock(pub NvmeInterface::<DmaAllocatorImpl, IrqControllerImpl>);

impl BlockDevice for VirtIOBlock {
    fn read_block(&mut self, sector_offset: usize, buf: &mut [u8]) {
        assert_eq!(buf.len(), 512);
        // 读取文件
        self.0.read_block(sector_offset, buf)
    }

    fn write_block(&mut self, sector_offset: usize, buf: &[u8]) {
        assert_eq!(buf.len(), 512);
        self.0.write_block(sector_offset, buf)
    }

    fn handle_irq(&mut self) {
        todo!()
    }
}

use core::ptr::write_volatile;

use crate::mutex::Mutex;

use super::{BlockDevice, DEVICE, DEVICE1};

// config pci
pub fn config_pci(){
    let ptr = 0x30008010 as *mut u32;
    unsafe { write_volatile(ptr, 0xffffffff); }
    let ptr = 0x30008010 as *mut u32;
    unsafe { write_volatile(ptr, 0x4); }
    let ptr = 0x30008010 as *mut u32;
    unsafe { write_volatile(ptr, 0x40000000); }
    let ptr = 0x30008004 as *mut u32;
    unsafe { write_volatile(ptr, 0x100006); }
    let ptr = 0x3000803c as *mut u32;
    unsafe { write_volatile(ptr, 0x21); }
}

pub fn init() {
    // 初始化 pci
    config_pci();

    unsafe {
        // 创建存储设备
        DEVICE.call_once(|| {
            let device = Box::new(VirtIOBlock(
                NvmeInterface::<DmaAllocatorImpl, IrqControllerImpl>::new(0x40000000)
            ));
            Mutex::new(device)
        });
    }
}