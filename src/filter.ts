import type { IGame, TimeClass } from './types' 
export function isUserWhite(username: string, game: IGame): boolean {
  return game.white.username === username
}

export function isUserBlack(username: string, game: IGame): boolean {
  return game.black.username === username
}

export function filterUserGamesByTimeControlAndPieces(
  username: string,
  pieces: 'black' | 'white',
  timeClass: TimeClass
): (game: IGame) => boolean {
  return function (game: IGame): boolean {
    if (game.time_class !== timeClass) {
      return false
    }
    if (pieces === 'black') {
      return isUserBlack(username, game)
    }
    if (pieces === 'white') {
      return isUserWhite(username, game)
    }

    return false
  }
}
