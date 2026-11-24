import styled, { keyframes } from 'styled-components';

const drift = keyframes`
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
`;

const gridMove = keyframes`
  from { background-position: 0 0, 0 0; }
  to { background-position: 0 60px, 0 0; }
`;

const pulse = keyframes`
  0%, 100% { transform: translateX(-50%) scale(1); opacity: 0.85; }
  50% { transform: translateX(-50%) scale(1.05); opacity: 1; }
`;

export const Window = styled.div`
  width: 100%;
  height: 100vh;
  background-color: #0f1923;
  overflow: hidden;

  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;

  .background {
    position: absolute;
    inset: 0;
    background: linear-gradient(120deg, #1b0a3c, #5b1a6e, #c2347a, #ff7a45, #5b1a6e, #1b0a3c);
    background-size: 300% 300%;
    animation: ${drift} 24s ease infinite;
    filter: brightness(0.5) grayscale(20%);

    .sun {
      position: absolute;
      left: 50%;
      bottom: 22%;
      width: 360px;
      height: 360px;
      border-radius: 50%;
      background: linear-gradient(#ffd36e, #ff4f8b);
      filter: blur(2px);
      animation: ${pulse} 8s ease-in-out infinite;
      transform: translateX(-50%);
    }

    .grid {
      position: absolute;
      left: 0;
      right: 0;
      bottom: 0;
      height: 36%;
      background:
        repeating-linear-gradient(0deg, #03c2fc55 0 2px, transparent 2px 60px),
        repeating-linear-gradient(90deg, #03c2fc33 0 2px, transparent 2px 80px);
      background-color: #12052a;
      animation: ${gridMove} 3s linear infinite;
    }
  }

  .drag {
    position: absolute;
    top: 0;
    left: 0;
    right: 150px;
    height: 40px;
    z-index: 1;
    -webkit-app-region: drag;
  }

  header {
    width: 100%;
    display: flex;
    flex-direction: column;
    padding: 40px;
    position: absolute;
    left: 0;
    top: 0;
    pointer-events: none;

    .user {
      display: flex;
      align-items: flex-end;
      width: 100%;
      z-index: 2;
      color: #fff;
      font-size: 20px;
      cursor: default;

      h3 {
        margin-right: 8px;
        max-width: 420px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      span {
        color: #aaa;
        font-weight: 700;
      }

      .level {
        margin-left: 12px;
        color: #03c2fc;
        font-size: 13px;
      }
    }

    .anchor {
      margin-bottom: 6px;
      font-size: 10px;
      font-weight: 700;
      z-index: 2;
      color: #03c2fc;
      cursor: pointer;
      pointer-events: auto;
      width: fit-content;

      &.warn {
        color: #ff4f8b;
      }

      &:hover {
        filter: brightness(0.9);
      }
    }
  }

  .window-options {
    display: flex;
    height: 50px;
    z-index: 3;

    position: absolute;
    right: 0;
    top: 0;
    -webkit-app-region: no-drag;

    .option {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 50px;
      height: 40px;

      cursor: pointer;

      &:hover,
      &:focus {
        background-color: #fff4;
      }

      &:hover:last-child,
      &:focus:last-child {
        background-color: #e34b49;
      }

      span {
        color: #fff;
        font-weight: 300;
      }
    }
  }

  .play {
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
    margin-top: -150px;

    h1 {
      font-size: 190px;
      color: #fff;
      cursor: pointer;
      transition: all 0.3s;
      font-weight: 900;
      letter-spacing: 2px;
      text-shadow: 0 4px 30px #0008;
    }

    svg {
      cursor: pointer;
      transition: all 0.3s;
    }

    &:hover {
      h1 {
        color: #ff4f8b;
      }
    }

    &.busy {
      h1 {
        color: #ffffff99;
        cursor: default;
      }
      svg {
        opacity: 0.5;
        cursor: default;
      }
    }
  }

  .launch-status {
    position: absolute;
    top: 400px;
    left: 0;
    right: 0;
    z-index: 2;
    text-align: center;
    color: #fff;
    font-size: 12px;
    letter-spacing: 1px;
    height: 16px;
  }

  .logo {
    position: absolute;
    bottom: 10px;
    left: 30px;
    width: 200px;
    z-index: 2;
    user-select: none;
    -webkit-user-drag: none;
  }

  .config {
    position: absolute;
    bottom: 40px;
    right: 40px;
    z-index: 3;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 7px;
    border-radius: 4px;
    background: #fff6;

    cursor: pointer;

    &:hover {
      background: #fffa;
    }
  }

  .side-panel,
  .options-panel {
    position: absolute;
    right: 30px;
    z-index: 2;
    width: 240px;
    background: #0008;
    border-radius: 6px;
    padding: 14px;
    color: #fff;

    h4 {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 11px;
      letter-spacing: 1px;
      color: #aaa;
      margin-bottom: 10px;

      &.spaced {
        margin-top: 14px;
      }
    }
  }

  .side-panel {
    top: 110px;

    .progress-label {
      display: flex;
      justify-content: space-between;
      font-size: 11px;
      color: #ccc;
      margin: 8px 0 4px;

      b {
        color: #fff;
      }
    }

    .bar {
      height: 6px;
      border-radius: 3px;
      background: #fff2;
      overflow: hidden;

      div {
        height: 100%;
        background: linear-gradient(90deg, #03c2fc, #ff4f8b);
        transition: width 0.4s;
      }
    }

    .mini {
      font-size: 10px;
      color: #888;
      margin-top: 4px;
    }

    .links {
      list-style: none;

      li {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 12px;
        padding: 5px 0;
        cursor: pointer;
        color: #03c2fc;

        &:hover {
          filter: brightness(0.85);
        }
      }
    }
  }

  .options-panel {
    bottom: 95px;
    width: 260px;

    .mini {
      font-size: 10px;
      color: #888;
      margin-top: 8px;
    }

    .path {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .row-opt {
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 12px;
      margin-bottom: 10px;

      select {
        background: #fff2;
        color: #fff;
        border-radius: 4px;
        padding: 4px 6px;
        font-family: inherit;
        font-size: 12px;

        option {
          color: #000;
        }
      }
    }

    .check {
      display: flex;
      align-items: center;
      font-size: 12px;
      margin-bottom: 8px;
      cursor: pointer;
      color: #ccc;

      &.disabled {
        opacity: 0.4;
        cursor: default;
      }

      input {
        display: none;
      }

      .box {
        width: 15px;
        height: 15px;
        border-radius: 2px;
        background-color: #fff3;
        margin-right: 10px;
      }

      input:checked + .box {
        background-color: #ff4f8b;
      }
    }
  }

  .log-panel {
    position: absolute;
    left: 30px;
    bottom: 55px;
    width: 700px;
    height: 190px;
    z-index: 2;
    background: #000a;
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    overflow: hidden;

    .log-head {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 8px 12px;
      font-size: 11px;
      letter-spacing: 1px;
      color: #aaa;
      font-weight: 700;
    }

    .log-actions {
      display: flex;
      gap: 8px;
    }

    .pill {
      display: flex;
      align-items: center;
      gap: 6px;
      font-family: inherit;
      font-size: 10px;
      font-weight: 700;
      letter-spacing: 1px;
      color: #fff;
      background: #03c2fc;
      padding: 5px 12px;
      border-radius: 14px;
      cursor: pointer;

      &:hover {
        filter: brightness(0.9);
      }

      &:disabled {
        opacity: 0.4;
        cursor: default;
      }

      &.danger {
        background: #e34b49;
      }
    }

    .log-body {
      flex: 1;
      overflow-y: auto;
      padding: 0 12px 10px;
      font-family: Consolas, 'Courier New', monospace;
      font-size: 11px;
      line-height: 1.5;

      .line {
        color: #ccc;
        white-space: pre-wrap;
        word-break: break-all;

        &.err {
          color: #ff7b7b;
        }

        &.info {
          color: #03c2fc;
        }

        &.game {
          color: #b9c4cf;
        }
      }
    }
  }
`;
