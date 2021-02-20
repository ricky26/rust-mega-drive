MEMORY
{
    ROM (rx) : ORIGIN = 0, LENGTH = 0x400000
    RAM (rwx) : ORIGIN = 0xFF0000, LENGTH = 0x10000
}

SECTIONS
{
    .text :
    {
        *(.text .text.*);
    } > ROM

    .rodata :
    {
        *(.rodata .rodata.*);
        _data_src = .;
    } > ROM

    .data :
    {
        ALIGN(4);
        _data_start = .;
        *(.data);
        ALIGN(4);
        _data_end = .;
    } > RAM AT > ROM

    .bss (NOLOAD) :
    {
        ALIGN(4);
        _bss_start = .;
        *(.bss);
        ALIGN(4);
        _bss_end = .;
    } > RAM AT > ROM

    _heap_start = .;
    _heap_end = 0x1000000;
}
