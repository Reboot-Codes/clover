MEMORY {
    /*
     * Raspberry Pi Pico 2 Flash
     * The standard Pico 2 has 4 MiB (4096K) of external QSPI flash.
     */
    FLASH : ORIGIN = 0x10000000, LENGTH = 4096K

    /*
     * RP2350 SRAM (520 KiB total)
     *
     * RAM consists of 8 banks, SRAM0-SRAM7 (512 KiB), with a striped mapping.
     * This distributes access evenly across banks for optimal concurrent performance.
     */
    RAM   : ORIGIN = 0x20000000, LENGTH = 512K

    /*
     * RAM banks 8 and 9 (4 KiB each) use a direct (non-striped) mapping.
     * Useful for predictable access times or dedicated core stacks (e.g. core1 stack).
     */
    SRAM8 : ORIGIN = 0x20080000, LENGTH = 4K
    SRAM9 : ORIGIN = 0x20081000, LENGTH = 4K
}

SECTIONS {
    /* ### Boot ROM info (IMAGE_DEF)
     *
     * Must be located in the first 4 KiB of flash so the RP2350 Boot ROM
     * and picotool can locate the image definition header.
     */
    .start_block : ALIGN(4)
    {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH
} INSERT AFTER .vector_table;

/* Move .text to start after the boot block */
_stext = ADDR(.start_block) + SIZEOF(.start_block);

SECTIONS {
    /* ### Picotool 'Binary Info' Entries
     *
     * Contains metadata entries inspected by picotool.
     */
    .bi_entries : ALIGN(4)
    {
        __bi_entries_start = .;
        KEEP(*(.bi_entries));
        . = ALIGN(4);
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    /* ### Boot ROM extra info / signature block */
    .end_block : ALIGN(4)
    {
        __end_block_addr = .;
        KEEP(*(.end_block));
    } > FLASH
} INSERT AFTER .uninit;

PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);
