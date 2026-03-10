ifeq ($(strip $(OLED_ENABLE)), yes)
	SRC += keyboards/mechboards/common/display_oled.c
endif
ifeq ($(strip $(QUANTUM_PAINTER_ENABLE)), yes)
	SRC += keyboards/mechboards/common/qp_font/font_small.qff.c
	SRC += keyboards/mechboards/common/qp_font/font_large.qff.c
	SRC += keyboards/mechboards/common/qp_font/font_huge.qff.c
endif
