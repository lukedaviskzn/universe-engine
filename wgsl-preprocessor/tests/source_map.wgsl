//!include("include/sub_include.wgsl")
//!define("A", "")
//!ifneq("A", "")
    TEST_G // don't include
//!else
    //!define("B", "ABC")
    TEST_H // include
    //!ifneq("B", "ABCDEF")
        TEST_O // include
        //!include("include/sub_include.wgsl")
    //!else
        TEST_P // don't include
        //!include("include/sub_include.wgsl")
    //!endif
//!endif
//!define("DEF", "d")
//!include("include/def_include.wgsl")
