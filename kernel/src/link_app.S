
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 4
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_3_end

    .global _app_names
_app_names:
    .string "cat"
    .string "echo"
    .string "hello_world"
    .string "test_read"

    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "../user/target/aarch64-unknown-none-softfloat/release/cat"
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
    .align 3
app_1_start:
    .incbin "../user/target/aarch64-unknown-none-softfloat/release/echo"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
    .align 3
app_2_start:
    .incbin "../user/target/aarch64-unknown-none-softfloat/release/hello_world"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
    .align 3
app_3_start:
    .incbin "../user/target/aarch64-unknown-none-softfloat/release/test_read"
app_3_end:
