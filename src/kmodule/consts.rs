pub const SHN_UNDEF     :u16 = 0;
pub const SHN_COMMON    :u16 = 0xfff2;

// symbol table bindings
pub const STB_LOCAL     :u8 = 0;
pub const STB_GLOBAL    :u8 = 1;
pub const STB_WEAK      :u8 = 2;
pub const STB_LOPROC    :u8 = 13;
pub const STB_HIPROC    :u8 = 15;

// symbol table type
pub const STT_NOTYPE    :u8 = 0;
pub const STT_OBJECT    :u8 = 1;
pub const STT_FUNC      :u8 = 2;
pub const STT_SECTION   :u8 = 3;
pub const STT_FILE      :u8 = 4;
pub const STT_LOPROC    :u8 = 13;
pub const STT_HIPROC    :u8 = 15;

/* values for Proghdr::p_type */
pub const ELF_PT_LOAD   :u8 = 1;

/* flag bits for Proghdr::p_flags */
pub const ELF_PF_X      :u8 = 1;
pub const ELF_PF_W      :u8 = 2;
pub const ELF_PF_R      :u8 = 4;

pub const MOD_INIT_MODULE   : &'static str = "init_module";
pub const MOD_CLEANUP_MODULE: &'static str = "cleanup_module";

pub const PGSIZE        :u64 = 4096;