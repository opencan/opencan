#include "signal.h"

#include <iostream>
#include <memory>
#include <sstream>
#include <string>

CANSignal::CANSignal(std::string_view name, int offset, int length) {
    if (offset < 0) {
        std::cerr << "Invalid offset while constructing signal " << name << ": " << offset << " is less than 0\n";

        std::exit(-3);
    }

    this->name = name;
    this->offset = offset;
    this->value = std::make_unique<CANValue>(length);
}

std::string CANSignal::printSummary() {
    std::stringstream s;
    s << "    Signal Name: " << this->name << '\n';
    s << "           Length: " << this->value->getLength() << '\n';
    s << "           Offset: " << this->offset << '\n';
    return s.str();
}
