use alloc::{boxed::Box};
use core::ptr::NonNull;
use core::sync::atomic::*;
use lazy_static::lazy_static;
use virtio_drivers::{Hal, MmioTransport, PhysAddr, VirtAddr, VirtIOBlk, VirtIOHeader};
use crate::mutex::Mutex;

use super::{DEVICE, BlockDevice, DEVICE1};

extern "C" {
    fn end();
}

lazy_static! {
    static ref DMA_PADDR: AtomicUsize = AtomicUsize::new(end as usize);
}

pub struct HalImpl;

impl Hal for HalImpl {
    fn dma_alloc(pages: usize) -> PhysAddr {
        let paddr = DMA_PADDR.fetch_add(0x1000 * pages, Ordering::SeqCst);
        println!("alloc DMA: paddr={:#x}, pages={}", paddr, pages);
        paddr
    }

    fn dma_dealloc(paddr: PhysAddr, pages: usize) -> i32 {
        println!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
        0
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        paddr
    }

    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        vaddr
    }
}

pub struct VirtIOBlock(pub VirtIOBlk::<HalImpl, MmioTransport>);

impl BlockDevice for VirtIOBlock {
    fn read_block(&mut self, sector_offset: usize, buf: &mut [u8]) {
        self.0.read_block(sector_offset, buf).expect("read error");
    }

    fn write_block(&mut self, sector_offset: usize, buf: &[u8]) {
        self.0.write_block(sector_offset, buf).expect("write error");
    }

    fn handle_irq(&mut self) {
        todo!()
    }
}

pub fn init() {
    unsafe {
        DEVICE.call_once(|| {
            let header = NonNull::new(0x10001000 as *mut VirtIOHeader).unwrap();
            let transport = unsafe { MmioTransport::new(header) }.unwrap();
            let device = VirtIOBlk::<HalImpl, MmioTransport>::new(transport)
                .expect("failed to create blk driver");
            Mutex::new(Box::new(VirtIOBlock(device)))
        });
    }
}