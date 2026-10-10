# The agent (DIRECTION P3.3, ADR-083 decision 4): QQ Studio's tools, offered to language models over MCP.
#
#   qqmcp     the protocol and the local pipe it is served on (Qt Core and Network only)
#   qqagent   the Studio's tools: what an agent can read and do in the Studio's window
#   qq-mcp    the program an MCP client starts: its standard input and output, relayed to the running Studio

qt_add_library(qqmcp STATIC mcpserver.h mcpserver.cpp)
target_include_directories(qqmcp PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qqmcp PUBLIC Qt6::Core Qt6::Network)

qt_add_library(qqagent STATIC studiotools.h studiotools.cpp)
target_link_libraries(qqagent PUBLIC qqmcp Qt6::Gui Qt6::Qml Qt6::Quick qqruntime)

# A console program: an MCP client reads its standard output.
qt_add_executable(qq-mcp bridge.cpp)
target_link_libraries(qq-mcp PRIVATE qqmcp)
