// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { OverlayMode } from './OverlayMode';

export type OverlayState = {
  overlay_editable: boolean;
  /**
   * stored as an integer representing percentage. Valid values are 0-100
   */
  overlay_opacity: number;
  overlay_mode: OverlayMode | null;
};
