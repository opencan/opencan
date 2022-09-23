#include "value.h"

#include <iostream>

CANValue::CANValue(int length) {
    if (length < 1) {
        std::cerr << "Invalid length while constructing value" << ": " << length << " is less than 1\n";

        std::exit(-3);
    }

    this->length = length;
}
