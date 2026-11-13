const en = {
  common: {
    loading: 'Loading…',
    loadFailed: 'Could not load',
    cancel: 'Cancel',
    confirm: 'Confirm',
  },
  language: {
    switch: 'Passer en français',
  },
  keyboard: {
    switch: 'Switch keyboard layout',
    azerty: 'AZERTY',
    qwerty: 'QWERTY',
  },
  hotkeys: {
    legend: 'Keyboard shortcuts',
    shape: 'shape',
    colour: 'colour',
    rotate: 'rotate',
    remove: 'remove',
    rightClick: 'Right-click',
    space: 'Space',
  },
  login: {
    description: 'Sign in to find your games.',
    login: 'Login',
    submit: 'Sign in',
    failed: 'Could not sign in',
  },
  home: {
    title: 'My games',
    signOut: 'Sign out',
    newGame: 'New game',
    createFailed: 'Could not create the game',
    empty: 'No games yet. Start one!',
  },
  status: {
    in_progress: 'In progress',
    solved: 'Solved',
    surrendered: 'Surrendered',
  },
  game: {
    title: 'Game',
    back: 'Back to games',
    surrender: 'Surrender',
    surrenderTitle: 'Surrender this game?',
    surrenderDescription: 'The hidden pieces will be revealed and firing will be disabled.',
    surrenderFailed: 'Could not surrender',
    shotFailed: 'Could not fire',
    offline: 'Not connected to the game: try again in a moment',
    reconnecting: 'Connection lost, reconnecting…',
    solved: 'Game solved: here are the hidden pieces. Firing is disabled.',
    surrendered: 'Game surrendered: here are the hidden pieces. Firing is disabled.',
    grid: 'Grid',
    gridRevealed: 'Grid and hidden pieces',
    removePiece: 'Right-click or Space to remove the {{shape}} piece',
    entry: 'Entry {{entry}}',
    entryShot: 'Entry {{entry}}, beam {{label}}: {{entry}} → {{other}}, {{colour}}',
  },
  shots: {
    title: 'Shots',
    empty: 'Click an entry around the grid to fire a beam.',
  },
  draft: {
    title: 'Draft',
    propose: 'Propose',
    proposeFailed: 'Could not propose',
    placed: '{{count}} / {{total}} pieces placed',
    incorrect: 'Incorrect',
    pieceToPlace: 'Piece to place',
    placedSuffix: ' (placed)',
    rotate: 'Rotate',
    rotateHint: 'Rotate a quarter turn (R)',
    colour: 'Colour',
    shapeHint: '{{shape}} ({{previous}} / {{next}})',
    colourHint: '{{colour}} ({{previous}} / {{next}})',
    cell: 'Cell {{x}}, {{y}}',
    cellPiece: 'Cell {{x}}, {{y}}: {{shape}} piece',
    cellPlace: 'Cell {{x}}, {{y}}: place the {{shape}} piece',
  },
  shapes: {
    square: 'square',
    bar: 'bar',
    spike: 'spike',
    corner: 'corner',
    diamond: 'diamond',
  },
  colours: {
    none: 'None',
    white: 'White',
    red: 'Red',
    blue: 'Blue',
    yellow: 'Yellow',
    violet: 'Violet',
    orange: 'Orange',
    green: 'Green',
    black: 'Black',
  },
}

export default en

type DeepStringify<T> = {
  [K in keyof T]: T[K] extends string ? string : DeepStringify<T[K]>
}

export type Translations = DeepStringify<typeof en>
