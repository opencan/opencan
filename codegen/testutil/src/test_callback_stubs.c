#include <stdint.h>

void CAN_callback_enqueue_tx_message(const uint8_t * const data, const uint8_t len, const uint32_t id) {
    (void)data;
    (void)len;
    (void)id;
}

uint64_t CAN_callback_get_system_time(void) {
    return 0;
}
