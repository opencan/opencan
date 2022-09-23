#pragma once

class CANValue {
    int length;
    public:
        int getLength() { return this->length; }
        CANValue(int length);
};
