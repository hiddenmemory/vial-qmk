// Copyright 2025 Dasky (@daskygit)
// SPDX-License-Identifier: GPL-2.0-or-later

#include "backlight.h"
#include "debug.h"
#include "keyboard.h"
#include "print.h"
#include "qp_surface_internal.h"
#include "quantum.h"
#include "common/display_lcd.h"
#include "rgb_matrix.h"

void keyboard_post_init_rs(void);
void raw_hid_receive_rs(uint8_t *data, uint8_t length);
void housekeeping_task_user_rs(void);
void rgb_matrix_indicators_advanced_rs(uint8_t led_min, uint8_t led_max);
bool process_record_user_rs(uint16_t keycode, bool pressed, keyrecord_t *record);

// We use this when we render directly to the surface, this reduces us needing to allocate
void r2g_surface_dirty_area(painter_device_t device, uint16_t l, uint16_t t, uint16_t r, uint16_t b) {
    surface_painter_device_t *surface = (surface_painter_device_t *)device;
    surface_dirty_data_t *dirty = &surface->dirty;

    dirty->l = MIN(l, dirty->l);
    dirty->t = MIN(t, dirty->t);
    dirty->b = MAX(b, dirty->b);
    dirty->r = MAX(r, dirty->r);
}

void debug_toggle(bool on) {
    debug_enable = on;
}

void keyboard_post_init_kb(void) {
    debug_toggle(true);
    keyboard_post_init_rs();
    keyboard_post_init_user();
}

void housekeeping_task_user() {
    housekeeping_task_user_rs();
}

void raw_hid_receive_kb(uint8_t *data, uint8_t length) {
    raw_hid_receive_rs(data, length);
}

bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max) {
    rgb_matrix_indicators_advanced_rs(led_min, led_max);
    return false;
}

bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    // For some reason, and it's too late for me to work out, but record->event.pressed
    // isn't able to be correctly read over the rust side, so less pull it out as an
    // official parameter
    return process_record_user_rs(keycode, record->event.pressed, record);
}

const pin_t row_pins_left[MATRIX_ROWS/2]  = MATRIX_ROW_PINS;
const pin_t row_pins_right[MATRIX_ROWS/2] = MATRIX_ROW_PINS_RIGHT;
const pin_t col_pins_left[MATRIX_COLS]  = MATRIX_COL_PINS;
const pin_t col_pins_right[MATRIX_COLS] = MATRIX_COL_PINS_RIGHT;

const uint32_t       *row_pins          = NULL;
const uint32_t       *col_pins          = NULL;

void matrix_init_kb(void){

    gpio_set_pin_input_high(ENCODER_SW_PIN);

    if (is_keyboard_left()) {
        row_pins = row_pins_left;
        col_pins = col_pins_left;
    } else {
        row_pins = row_pins_right;
        col_pins = col_pins_right;
    }
    matrix_init_user();
}

void matrix_output_unselect_delay(uint8_t line, bool key_pressed) {
    if (key_pressed){
        bool done = false;
        while (done == false) {
            bool cols_high = true;
            for (uint8_t col_index = 0; col_index < MATRIX_COLS; col_index++) {
                if (gpio_read_pin(col_pins[col_index]) == 0){
                    cols_high = false;
                }
            }
            if (cols_high){
                done = true;
            }
        }
    }
}

void matrix_read_cols_on_row(matrix_row_t current_matrix[], uint8_t current_row) {
    // Start with a clear matrix row
    matrix_row_t current_row_value = 0;

    ATOMIC_BLOCK_FORCEON {
        gpio_set_pin_output(row_pins[current_row]);
        gpio_write_pin_low(row_pins[current_row]);
    }
    while (gpio_read_pin(row_pins[current_row])!= 0){

    };

    // For each col...
    matrix_row_t row_shifter = MATRIX_ROW_SHIFTER;
    for (uint8_t col_index = 0; col_index < MATRIX_COLS; col_index++, row_shifter <<= 1) {
        uint8_t pin_state = gpio_read_pin(col_pins[col_index]);

        if (col_index == (is_keyboard_left() ? 5:0) && current_row == 4){
            if (gpio_read_pin(ENCODER_SW_PIN) == 0){
                pin_state = 0;
            }
        }

        current_row_value |= pin_state ? 0 : row_shifter;
    }

    // Unselect row
    ATOMIC_BLOCK_FORCEON {
        gpio_set_pin_input_high(row_pins[current_row]);
    }
    matrix_output_unselect_delay(current_row, current_row_value != 0); // wait for all Col signals to go HIGH

    // Update the matrix
    current_matrix[current_row] = current_row_value;
}

