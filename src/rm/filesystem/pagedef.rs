// 一个页面中的字节数
pub const PAGE_SIZE: i32 = 8192;

// 一个页面中的整数个数
pub const PAGE_INT_NUM: i32 = 2048;

// 页面字节数以2为底的指数
pub const PAGE_SIZE_IDX: i32 = 13;

const MAX_FMT_INT_NUM: i32 = 128;

pub const MAX_FILE_NUM: usize = 128;
pub const MAX_TYPE_NUM: usize = 256;

// 缓存中页面个数上限
pub const CAP: usize =  60000;

// hash算法的模
pub const MOD: usize =  60000;

const IN_DEBUG: i32 =  0;
const DEBUG_DELETE: i32 =  0;
const DEBUG_ERASE: i32 =  1;
const DEBUG_NEXT: i32 =  1;
//  一个表中列的上限
const MAX_COL_NUM: i32 =  31;

//  数据库中表的个数上限
const MAX_TB_NUM: i32 =  31;
const RELEASE: i32 =  1;


// typedef int(cf)(uchar*, uchar*);
// static mut current: i32 =  0;
// static mut tt: i32 =  0;
