//!define("ABC", "B")
ABC // ABC should be B
/*
ABC, should be B
*/
//!define("B", "C")
ABC B C // ABC B C, should be B C C
/* ABC B C, should be B C C
*/
//!define("ABC", "ABC")
ABC B C // ABC B C, should be A C C
/*
ABC B C, should be A C C*/
