// ripex-lang-test: C++ service — namespace, RAII, async-ish futures.
#include <thread>
#include <future>
#include <vector>
#include <string>

namespace ripex::services {

std::vector<std::string> fetch_all(const std::vector<std::string>& urls) {
    std::vector<std::future<std::string>> tasks;
    for (const auto& u : urls) {
        tasks.push_back(std::async(std::launch::async, [&u]() { return u; }));
    }
    std::vector<std::string> out;
    for (auto& t : tasks) {
        out.push_back(t.get());
    }
    return out;
}

void log(const std::string& msg) {
    // logging
}

} // namespace ripex::services
