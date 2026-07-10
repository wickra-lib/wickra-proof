# Build and run the C and C++ examples against the wickra-proof C ABI.
#
#   cargo build --release -p wickra-proof-c
#   cmake -S examples/c -B examples/c/build
#   cmake --build examples/c/build --config Release
#   ctest --test-dir examples/c/build -C Release --output-on-failure
#
# WICKRA_PROOF_LIB_DIR defaults to the workspace target/release directory and can
# be overridden for out-of-tree builds.

cmake_minimum_required(VERSION 3.15)
project(wickra_proof_examples C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(WICKRA_PROOF_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release"
    CACHE PATH "Directory containing the built wickra-proof C ABI library")

set(WICKRA_PROOF_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/../../bindings/c/include")

find_library(WICKRA_PROOF_LIB
    NAMES wickra_proof libwickra_proof
    HINTS "${WICKRA_PROOF_LIB_DIR}"
    REQUIRED)

add_executable(prove prove.c)
target_include_directories(prove PRIVATE "${WICKRA_PROOF_INCLUDE}")
target_link_libraries(prove PRIVATE "${WICKRA_PROOF_LIB}")

add_executable(provepp prove.cpp)
target_include_directories(provepp PRIVATE "${WICKRA_PROOF_INCLUDE}")
target_link_libraries(provepp PRIVATE "${WICKRA_PROOF_LIB}")

# On Windows the runtime DLL must sit next to the executable (there is no rpath);
# without it the loader pops a modal "DLL not found" dialog that hangs ctest. Copy
# it next to each example after build. (Unix resolves the .so/.dylib via the
# linker's embedded path, so this is Windows-only.)
if(WIN32)
    foreach(example prove provepp)
        add_custom_command(TARGET ${example} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${WICKRA_PROOF_LIB_DIR}/wickra_proof.dll"
                "$<TARGET_FILE_DIR:${example}>"
            VERBATIM)
    endforeach()
endif()

enable_testing()
add_test(NAME prove COMMAND prove)
add_test(NAME provepp COMMAND provepp)
# A missing runtime dependency must fail fast, never block the runner for hours.
set_tests_properties(prove provepp PROPERTIES TIMEOUT 60)
