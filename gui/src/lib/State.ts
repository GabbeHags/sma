class State {
  config_path: string;
  starts: string[];
  cascade_kill: boolean;

  constructor() {
    this.config_path = '';
    this.starts = [''];
    this.cascade_kill = false;
  }
}

export default State;
