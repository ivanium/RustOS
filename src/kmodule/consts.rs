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

/* x86-64 relocation types */
pub const R_X86_64_NONE      :u32 = 0;       /* No reloc */
pub const R_X86_64_64        :u32 = 1;       /* Direct 64 bit  */
pub const R_X86_64_PC32      :u32 = 2;       /* PC relative 32 bit signed */
pub const R_X86_64_GOT32     :u32 = 3;       /* 32 bit GOT entry */
pub const R_X86_64_PLT32     :u32 = 4;       /* 32 bit PLT address */
pub const R_X86_64_COPY      :u32 = 5;       /* Copy symbol at runtime */
pub const R_X86_64_GLOB_DAT  :u32 = 6;       /* Create GOT entry */
pub const R_X86_64_JUMP_SLOT :u32 = 7;       /* Create PLT entry */
pub const R_X86_64_RELATIVE  :u32 = 8;       /* Adjust by program base */
pub const R_X86_64_GOTPCREL  :u32 = 9;       /* 32 bit signed pc relative                                          offset to GOT */
pub const R_X86_64_32        :u32 = 10;      /* Direct 32 bit zero extended */
pub const R_X86_64_32S       :u32 = 11;      /* Direct 32 bit sign extended */
pub const R_X86_64_16        :u32 = 12;      /* Direct 16 bit zero extended */
pub const R_X86_64_PC16      :u32 = 13;      /* 16 bit sign extended pc relative */
pub const R_X86_64_8         :u32 = 14;      /* Direct 8 bit sign extended  */
pub const R_X86_64_PC8       :u32 = 15;      /* 8 bit sign extended pc relative */