#include "signal.h"

#include <iostream>
#include <string>

int main() {
    std::cout << "Hello from protobrain." << '\n';

    auto s = new CANSignal("VCFRONT_driverIsLeaving", 1, 1);

    std::cout << "Signal details:\n" << s->printSummary();
}
