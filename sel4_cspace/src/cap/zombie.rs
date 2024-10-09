//! zombie cap相关字段和方法
//! 当`tcb_cap`和`cnode_cap`删除的过程中会变为`zombie_cap`
use crate::cte::cte_t;
use sel4_common::sel4_config::wordRadix;
use sel4_common::structures_gen::{cap, cap_Splayed, cap_tag, cap_zombie_cap};
use sel4_common::MASK;

use super::cap_t;

/// Judge whether the zombie cap is from tcb cap.
pub const ZombieType_ZombieTCB: usize = 1usize << wordRadix;
pub const TCB_CNODE_RADIX: usize = 4;

/// zombie cap相关字段和方法
pub trait cap_zombie_func {
	fn get_zombie_bit(&self) -> usize;
	fn get_zombie_ptr(&self) -> usize;
	fn get_zombie_number(&self) -> usize;
	fn set_zombie_number(&mut self, n: usize);
}
impl cap_zombie_func for cap_zombie_cap {
    #[inline]
    fn get_zombie_bit(&self) -> usize {
        let _type = self.get_capZombieType() as usize;
        if _type == ZombieType_ZombieTCB {
            return TCB_CNODE_RADIX;
        }
        ZombieType_ZombieCNode(_type)
    }

    #[inline]
    fn get_zombie_ptr(&self) -> usize {
        let radix = self.get_zombie_bit();
        self.get_capZombieID() as usize & !MASK!(radix + 1)
    }

    #[inline]
    fn get_zombie_number(&self) -> usize {
        let radix = self.get_zombie_bit();
        self.get_capZombieID() as usize & MASK!(radix + 1)
    }

    #[inline]
    fn set_zombie_number(&mut self, n: usize) {
        let radix = self.get_zombie_bit();
        let ptr = self.get_capZombieID() as usize & !MASK!(radix + 1);
        self.set_capZombieID((ptr | (n & MASK!(radix + 1))) as u64);
    }
}

#[inline]
pub fn Zombie_new(number: usize, _type: usize, ptr: usize) -> cap {
    let mask = if _type == ZombieType_ZombieTCB {
        MASK!(TCB_CNODE_RADIX + 1)
    } else {
        MASK!(_type + 1)
    };
	cap_zombie_cap::new(((ptr & !mask) | (number & mask)) as u64, _type as u64).unsplay()
}

pub fn ZombieType_ZombieCNode(n: usize) -> usize {
    n & MASK!(wordRadix)
}

///判断是否为循环`zombie cap`,指向自身且类型为`CapZombieCap`（似乎只有`CNode Capability`指向自己才会出现这种情况）
/// 根据网上信息，当`cnode cap`为L2以上时，即`CNode`嵌套`CNode`的情况，就会产生`CyclicZombie`
#[inline]
#[no_mangle]
pub fn capCyclicZombie(capability: &cap, slot: *mut cte_t) -> bool {
	match capability.clone().splay(){
		cap_Splayed::zombie_cap(data)=>{
			let ptr = data.get_zombie_ptr() as *mut cte_t;
			(capability.get_tag() == cap_tag::cap_zombie_cap) && (ptr == slot)
		}
		_=> false
	}
}
