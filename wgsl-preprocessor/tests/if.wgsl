//!define("A", "")
//!ifdef("A")
    TEST_A // include
//!else
    TEST_B // don't include
//!endif
//!ifndef("A")
    TEST_C // don't include
//!else
    TEST_D // include
//!endif
//!ifeq("A", "")
    TEST_E // include
//!else
    TEST_F // don't include
//!endif
//!ifneq("A", "")
    TEST_G // don't include
//!else
    TEST_H // include
    //!define("B", "ABC")
    //!ifdef("B")
        TEST_I // include
    //!else
        TEST_J // don't include
    //!endif
    //!ifndef("B")
        TEST_K // don't include
    //!else
        TEST_L // include
    //!endif
    //!ifeq("B", "ABC")
        TEST_M // include
    //!else
        TEST_N // don't include
    //!endif
    //!ifneq("B", "ABCDEF")
        TEST_O // include
        //!define("DEF", "d")
        //!include("include/def_include.wgsl")
    //!else
        TEST_P // don't include
        //!include("include/def_include.wgsl")
    //!endif
//!endif