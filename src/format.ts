import type { IGame, IFormattedGame, IOutputJSON, GameResult } from './types'
import { getGameAccuracy, getOpening, getGameResult } from './parse'

export function formatGames(username: string): (game: IGame) => IFormattedGame {
  return function mapGame(game: IGame): IFormattedGame {
    return {
      accuracy: getGameAccuracy(username, game),
      opening: getOpening(game),
      result: getGameResult(username, game)
    }
  }
}

function sortOpeningCounts(openingCounts: IOutputJSON): IOutputJSON {
  return Object.fromEntries(
    Object.entries(openingCounts).sort(([,a],[,b]) => b.count-a.count)
  )
}

export function groupByOpening(formattedGames: IFormattedGame[]): IOutputJSON {
  return sortOpeningCounts(formattedGames.reduce((acc, { accuracy, opening, result }) => {
    if (acc[opening]) {
      acc[opening].count++

      if (acc[opening].averageAccuracy && accuracy) {
        acc[opening].averageAccuracy = (acc[opening].averageAccuracy as number + accuracy) / 2
      } else if (accuracy) {
        acc[opening].averageAccuracy = accuracy
      }

      acc[opening].results[result] = acc[opening].results[result]
        ? acc[opening].results[result] + 1
        : 1

      return acc
    }

    acc[opening] = { count: 0, results: {} as Record<GameResult, number>}
    acc[opening].count = 1

    if (accuracy) {
      acc[opening].averageAccuracy = accuracy
    }
    acc[opening].results = {} as Record<GameResult, number>
    acc[opening].results[result] = 1
    return acc
  }, {} as IOutputJSON))
}
