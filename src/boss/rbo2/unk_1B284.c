// SPDX-License-Identifier: AGPL-3.0-or-later
#include "rbo2.h"
#include "sfx.h"

extern EInit D_us_801804A0;
#ifdef VERSION_PSP
extern s32 D_pspeu_09265CC8;
#define BOSS_FLAGS D_pspeu_09265CC8
#else
extern s32 D_us_80180B5C;
#define BOSS_FLAGS D_us_80180B5C
#endif

static u8 s_OrbAppearAnim[] = {
    0x09, 0x1D, 0x04, 0x1E, 0x04, 0x1F, 0x04, 0x20, 0x03, 0x21, 0x03, 0x22,
    0x03, 0x23, 0x03, 0x24, 0x03, 0x25, 0x04, 0x26, 0x08, 0x27, 0xFF, 0x00,
};

static u8 s_OrbBreakAnim[] = {
    0x03, 0x25, 0x03, 0x28, 0x03, 0x29, 0x03, 0x2A,
    0x03, 0x1F, 0x04, 0x1E, 0x05, 0x1D, 0xFF, 0x00,
};

void func_us_8019C718(Entity* self) {
    Entity* player;
    s32 playerX;
    s32 playerY;
    s16 angle;

    if (BOSS_FLAGS & 8) {
        self->flags |= FLAG_DEAD;
    }
    if ((self->flags & FLAG_DEAD) && self->step != 3) {
        PlaySfxPositional(SFX_METAL_CLANG_C);
        self->hitboxState = 0;
        SetStep(3);
    }

    switch (self->step) {
    case 0:
        InitializeEntity(D_us_801804A0);
        self->facingLeft = Random() & 1;
        self->hitboxState = 0;
        // fallthrough

    case 1:
        if (AnimateEntity(s_OrbAppearAnim, self) != 0) {
            return;
        }
        self->hitboxState = 3;
        self->animCurFrame = 0x4F;
        self->drawFlags = ENTITY_ROTATE;
        SetStep(2);
        return;

    case 2:
        if (!self->step_s) {
            player = &PLAYER;
            playerX = player->posX.i.hi;
            playerY = player->posY.i.hi;
            playerX += (Random() & 0x3F) - 0x20;
            playerY += (Random() & 0x3F) - 0x20;
            playerX -= self->posX.i.hi;
            playerY -= self->posY.i.hi;
            angle = ratan2(playerY, playerX);
            self->velocityX = (rcos(angle) << 16) >> 12;
            self->velocityY = (rsin(angle) << 16) >> 12;
            PlaySfxPositional(SFX_WEAPON_SCRAPE_ECHO);
            self->step_s++;
        }
        MoveEntity();
        self->rotate += 0xC0;
        return;

    case 3:
        if (AnimateEntity(s_OrbBreakAnim, self) == 0) {
            DestroyEntity(self);
        }
        break;
    }
}
