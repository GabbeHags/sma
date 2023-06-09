import type { IRustConfig } from '$lib/rust-bindings';

export class State {
  configPath: string;
  config: Config;

  constructor() {
    this.configPath = '';
    this.config = new Config();
  }
}

export class Config {
  version: number;
  cwd: string;
  cascadeKill: boolean;
  start: string[];
  exitOn: ExitOn;

  constructor(config: IRustConfig = {} as IRustConfig) {
    const { version = 1, cwd = null, cascadeKill = false, start = [''], exitOn = null } = config;

    let tempCwd: string;
    if (cwd === null) {
      tempCwd = '';
    } else {
      tempCwd = cwd;
    }
    const tempExitOn = new ExitOn();
    if (exitOn !== null) {
      tempExitOn.active = true;
      tempExitOn.num = exitOn;
      tempExitOn.num = exitOn;
    }
    this.version = version;
    this.cwd = tempCwd;
    this.cascadeKill = cascadeKill;
    this.start = start;
    this.exitOn = tempExitOn;
  }
}

class ExitOn {
  private _num: number;
  active: boolean;
  displayNum: number;

  constructor() {
    this.active = false;
    this.displayNum = 1;
    this._num = this.displayNum - 1;
  }

  set num(num: number) {
    this._num = num;
    this.displayNum = this._num + 1;
  }

  update_num() {
    this._num = Math.max(0, this.displayNum - 1);
  }

  clamp_display_num(min: number, max: number) {
    this.displayNum = Math.max(min, Math.min(this.displayNum, max));
  }
}
