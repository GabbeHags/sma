class State {
  config_path: string;
  starts: string[];
  cascade_kill: boolean;
  exit_on: ExitOn;

  constructor() {
    this.config_path = '';
    this.starts = [''];
    this.cascade_kill = false;
    this.exit_on = new ExitOn();
  }
}

class ExitOn {
  private _num: number;
  active: boolean;
  display_num: number;

  constructor() {
    this.active = false;
    this.display_num = 1;
    this._num = this.display_num - 1;
  }

  update_num() {
    this._num = Math.max(0, this.display_num - 1);
  }

  clamp_display_num(min: number, max: number) {
    this.display_num = Math.max(min, Math.min(this.display_num, max));
  }
}

export default State;
