import type { IGame, GameResult, IPGN } from './types'
import { isUserBlack, isUserWhite } from './filter'

function parsePGN(pgn: string): IPGN {
  let pgnJSON: IPGN = {
    tags: {},
    moves: ''
  }

  const lines = pgn.trim().split('\n')
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]

    if (/^\[/.test(line)) {
      const matches = line.match(/^\[([^\]]+)\]/)
      const tags = matches?.[1]
      if (tags) {
        const [key, value] = tags.split(' ')
        pgnJSON.tags[key] = value
      }
    }

    pgnJSON.moves = line
  }

  return pgnJSON
}

export function getOpening(game: IGame): string {
  const pgn = parsePGN(game.pgn)
  const openingUrlParts = pgn.tags.ECOUrl?.split('/')
  const opening = openingUrlParts?.[openingUrlParts?.length - 1]
  return opening
}

export function getGameResult(username: string, game: IGame): GameResult {
  if (isUserWhite(username, game)) {
    return game.white.result
  }
  if (isUserBlack(username, game)) {
    return game.black.result
  }

  return ''
}

export function getGameAccuracy(username: string, game: IGame): number | undefined {
  if (isUserWhite(username, game)) {
    return game.accuracies?.white
  }
  if (isUserBlack(username, game)) {
    return game.accuracies?.black
  }

  return undefined
}

