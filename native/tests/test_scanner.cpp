#include "test_main.hpp"
#include "memory_scanner.hpp"
#include <cstring>

namespace {

class MockProcessBackend final : public cheat_agent::IProcessBackend {
public:
    MockProcessBackend() {
        memory_.resize(0x10000, 0); // 64 KiB memory
    }

    [[nodiscard]] bool is_alive() const noexcept override { return true; }
    [[nodiscard]] uint32_t get_pid() const noexcept override { return 1337; }

    int32_t read_memory(uint64_t address, void* buffer, size_t size, size_t& bytes_read) override {
        bytes_read = 0;
        if (address + size > memory_.size()) return CHEAT_STATUS_ERROR;
        std::memcpy(buffer, memory_.data() + address, size);
        bytes_read = size;
        return CHEAT_STATUS_OK;
    }

    int32_t write_memory(uint64_t address, const void* buffer, size_t size, size_t& bytes_written) override {
        bytes_written = 0;
        if (address + size > memory_.size()) return CHEAT_STATUS_ERROR;
        std::memcpy(memory_.data() + address, buffer, size);
        bytes_written = size;
        return CHEAT_STATUS_OK;
    }

    std::vector<cheat_agent::MemoryRegion> query_regions() override {
        return {
            {0x0000, 0x10000, CHEAT_PROTECT_READ | CHEAT_PROTECT_WRITE, true, true, false}
        };
    }

    std::vector<cheat_agent::ModuleInfo> query_modules() override {
        return {
            {"mock_game.exe", "/bin/mock_game.exe", 0x1000, 0x2000}
        };
    }

    std::vector<uint8_t>& memory() { return memory_; }

private:
    std::vector<uint8_t> memory_;
};

} // namespace

TEST_CASE(memory_scanner_exact_and_decreased_refinement) {
    auto backend = std::make_shared<MockProcessBackend>();
    cheat_agent::MemoryScanner scanner(backend);

    // Write initial game state:
    // Ammo at 0x2040 = 100
    // Another variable at 0x3040 = 100
    uint32_t val100 = 100;
    size_t written = 0;
    backend->write_memory(0x2040, &val100, sizeof(val100), written);
    backend->write_memory(0x3040, &val100, sizeof(val100), written);

    // First scan: exact uint32 = 100
    int32_t res = scanner.first_scan(CHEAT_DATA_TYPE_UINT32, CHEAT_SCAN_MODE_EXACT, &val100, nullptr, 4);
    REQUIRE_EQ(res, CHEAT_STATUS_OK);
    REQUIRE(scanner.get_match_count() >= 2u);

    // Simulate ammo firing: ammo at 0x2040 decreases to 95, other variable stays 100
    uint32_t val95 = 95;
    backend->write_memory(0x2040, &val95, sizeof(val95), written);

    // Next scan: decreased
    res = scanner.next_scan(CHEAT_SCAN_MODE_DECREASED, nullptr, nullptr, 4);
    REQUIRE_EQ(res, CHEAT_STATUS_OK);

    // Only 0x2040 should match
    bool found_ammo = false;
    for (const auto& c : scanner.get_candidates()) {
        if (c.address == 0x2040) found_ammo = true;
        REQUIRE(c.address != 0x3040); // 0x3040 stayed 100, so it must not match decreased
    }
    REQUIRE(found_ammo);
}
