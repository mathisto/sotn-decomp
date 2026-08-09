// SPDX-License-Identifier: AGPL-3.0-or-later
#include "rbo2.h"
#include "sfx.h"

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", EntityBreakable);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019A98C_from_rcen);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019B430);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_801CDC80);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019B52C);

extern EInit D_us_801804A0;
extern u8 D_us_801806E0[];
extern u8 D_us_801806F8[];
extern s32 D_us_80180B5C;

void func_us_8019C718(Entity* self) {
    s32 offset;
    s32 playerY;
    s32 playerX;
    s16 angle;

    if (D_us_80180B5C & 8) {
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
        if (AnimateEntity(D_us_801806E0, self) == 0) {
            self->hitboxState = 3;
            self->animCurFrame = 0x4F;
            self->drawFlags = ENTITY_ROTATE;
            SetStep(2);
        }
        return;

    case 2:
        if (self->step_s == 0) {
            playerX = PLAYER.posX.i.hi;
            offset = 0x20;
            playerY = PLAYER.posY.i.hi;
            playerX = playerX - offset + (Random() & 0x3F);
            playerY = playerY - offset + (Random() & 0x3F);
            angle =
                ratan2(playerY - self->posY.i.hi, playerX - self->posX.i.hi);
            self->velocityX = (rcos(angle) << 16) >> 12;
            self->velocityY = (rsin(angle) << 16) >> 12;
            PlaySfxPositional(SFX_WEAPON_SCRAPE_ECHO);
            self->step_s++;
        }
        MoveEntity();
        self->rotate += 0xC0;
        return;

    case 3:
        if (AnimateEntity(D_us_801806F8, self) == 0) {
            DestroyEntity(self);
        }
        break;
    }
}

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019C924);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019D4CC);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019D950);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019DA04);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019E558);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019E920);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019ECCC);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019ED80);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", polarPlacePartsWithAngvel);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_801CDD00);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_801CDD80);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_801CDF1C);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_801CE1E8);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_801CE228);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", polarPlacePartsList);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019D260_from_rcen);

INCLUDE_ASM("boss/rbo2/nonmatchings/unk_1B284", func_us_8019F260);

extern EInit g_EInitInteractable;
extern s32 D_us_80180B5C;
extern u32 D_us_801AE900;

void func_us_8019F4AC(Entity* self) {
    Entity* entity;
    u32 posX;
    s32 posY;

    switch (self->step) {
    case 0:
        InitializeEntity(g_EInitInteractable);
        self->flags |= FLAG_UNK_10000;
        // fallthrough

    case 1:
        posX = PLAYER.posX.i.hi + g_Tilemap.scrollX.i.hi;
        if ((posX - 0x41) < 0x17F) {
            D_us_80180B5C |= 1;
            self->step++;
        }
        break;

    case 2:
        if (g_CastleFlags[DEATH_FIGHT_CS] ||
            (g_PlayableCharacter != PLAYER_ALUCARD) ||
            (g_DemoMode != Demo_None)) {
            posX = PLAYER.posX.i.hi + g_Tilemap.scrollX.i.hi;
            if ((posX - 0x81) >= 0xFF) {
                break;
            }
        } else if (!(D_us_801AE900 & 2)) {
            break;
        }
        g_api.TimeAttackController(
            TIMEATTACK_EVENT_DEATH_DEFEAT, TIMEATTACK_SET_VISITED);
        stopMusicFlag = true;
        currentMusicId = MU_DEATH_BALLAD;
        D_us_80180B5C |= 2;
        self->step++;
        break;

    case 3:
        if (g_api.func_80131F68() == false) {
            stopMusicFlag = false;
            g_api.PlaySfx(currentMusicId);
            self->step++;
        }
        // fallthrough

    case 4:
        if (D_us_80180B5C & 0x10) {
            g_api.TimeAttackController(
                TIMEATTACK_EVENT_DEATH_DEFEAT, TIMEATTACK_SET_RECORD);
            g_api.PlaySfx(SET_UNK_90);
            currentMusicId = MU_ABANDONED_PIT;
            self->step++;
        }
        break;

    case 5:
        if (D_us_80180B5C & 0x40) {
            self->step++;
        }
        break;

    case 6:
        posX = 0x100 - g_Tilemap.scrollX.i.hi;
        posY = 0x80 - g_Tilemap.scrollY.i.hi;
        entity = AllocEntity(&g_Entities[160], &g_Entities[192]);
        if (entity != NULL) {
            CreateEntityFromEntity(E_LIFE_UPSPAWN, self, entity);
            entity->posX.i.hi = posX;
            entity->posY.i.hi = posY;
            entity->params = 0x15;
            stopMusicFlag = true;
            currentMusicId = MU_ABANDONED_PIT;
            D_us_80180B5C |= 0x80;
            self->step++;
        }
        break;

    case 7:
        if (g_api.func_80131F68() == false) {
            stopMusicFlag = false;
            g_api.PlaySfx(currentMusicId);
            self->step++;
        }
        break;
    }

    FntPrint("set_step %x\n", self->step);
    FntPrint("boss_flag %x\n", D_us_80180B5C);
}
