import type { IRustConfig, IToRustConfig } from '$lib/rust-bindings';

export class State {
  configPath: string;
  config: Config;

  constructor() {
    this.configPath = '';
    this.config = new Config();
  }
}

export class Config implements IToRustConfig {
  version: number;
  cwd: string;
  cascadeKill: boolean;
  start: string[];
  exitOn: ExitOn;

  constructor(config: Config = {} as Config) {
    const {
      version = 1,
      cwd = '.',
      cascadeKill = false,
      start = [''],
      exitOn = new ExitOn()
    } = config;

    this.version = version;
    this.cwd = cwd;
    this.cascadeKill = cascadeKill;
    this.start = start;
    this.exitOn = new ExitOn(exitOn);
  }

  static fromRustConfig(rustConfig: IRustConfig = {} as IRustConfig): Config {
    const { version = 1, cwd = null, cascadeKill = false, start = [], exitOn = null } = rustConfig;
    const config = new Config();

    let tempCwd = '.';
    if (cwd !== null) {
      tempCwd = cwd;
    }

    const tempExitOn = new ExitOn();
    if (exitOn !== null) {
      tempExitOn.active = true;
      tempExitOn.num = exitOn;
    }

    let tempStart = [''];
    if (start.length > 0) {
      tempStart = start;
    }

    config.version = version;
    config.cwd = tempCwd;
    config.cascadeKill = cascadeKill;
    config.start = tempStart;
    config.exitOn = tempExitOn;
    return config;
  }

  cleanUpStart(): Config {
    const isEmpty = (s: string): boolean => s.replaceAll(' ', '') == '';
    const config: Config = new Config(this);
    config.exitOn.active = config.exitOn.active && !isEmpty(config.start[config.exitOn.num]);
    console.log(config);
    if (config.exitOn.active) {
      let leftHalf: string[] = [];
      let rightHalf: string[] = [];

      // cleanup all the items right of `num`
      if (config.start.length - 1 != config.exitOn.num) {
        rightHalf = config.start.slice(config.exitOn.num + 1, undefined).filter((x) => !isEmpty(x));
      }

      // cleanup all the items left and up to `num`
      leftHalf = config.start.slice(0, config.exitOn.num + 1).filter((x) => !isEmpty(x));

      // subtract the amount of items removed from start to exitOn
      config.exitOn.num -= config.exitOn.num + 1 - leftHalf.length;

      config.start = leftHalf.concat(rightHalf);
    } else {
      config.start = config.start.filter((x) => !isEmpty(x));
    }
    return config;
  }

  toRustConfig(): IRustConfig {
    const config = this.cleanUpStart();
    const exitOn: number | null = config.exitOn.active ? config.exitOn.num : null;

    return {
      version: config.version,
      cwd: config.cwd,
      cascadeKill: config.cascadeKill,
      start: config.start,
      exitOn
    };
  }
}

class ExitOn {
  private _num: number;
  active: boolean;
  displayNum: number;

  constructor(exitOn: ExitOn = {} as ExitOn) {
    const { active = false, displayNum = 1 } = exitOn;
    this.active = active;
    this.displayNum = displayNum;
    this._num = exitOn.num ? exitOn.num : displayNum - 1;
  }

  set num(num: number) {
    this._num = num;
    this.displayNum = this._num + 1;
  }
  get num(): number {
    return this._num;
  }

  update_num() {
    this._num = Math.max(0, this.displayNum - 1);
  }

  clamp_display_num(min: number, max: number) {
    this.displayNum = Math.max(min, Math.min(this.displayNum, max));
  }
}
