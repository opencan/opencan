#pragma once

#include "value.h"

#include <memory>
#include <string>

class CANSignal {
    private:
        std::string name;
        int offset;
        std::unique_ptr<CANValue> value;

    public:
        std::string_view getName() { return name; }
        CANSignal(std::string_view name, int offset, int length);

        std::string printSummary();
};
