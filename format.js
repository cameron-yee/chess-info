const { getGameAccuracy, getOpening, getGameResult } = require('./parse')

function formatGames(username) {
  return function mapGame(game) {
    return {
      accuracy: getGameAccuracy(username, game),
      opening: getOpening(game),
      result: getGameResult(username, game)
    }
  }
}

function sortOpeningCounts(openingCounts) {
  return Object.fromEntries(
    Object.entries(openingCounts).sort(([,a],[,b]) => b.count-a.count)
  )
}

function groupByOpening(formattedGames) {
  return sortOpeningCounts(formattedGames.reduce((acc, { accuracy, opening, result }) => {
    if (acc[opening]) {
      acc[opening].count++

      if (acc[opening].averageAccuracy && accuracy) {
        acc[opening].averageAccuracy = (acc[opening].averageAccuracy + accuracy) / 2
      } else if (accuracy) {
        acc[opening].averageAccuracy = accuracy
      }

      acc[opening].results[result] = acc[opening].results[result]
        ? acc[opening].results[result] + 1
        : 1

      return acc
    }

    acc[opening] = {}
    acc[opening].count = 1

    if (accuracy) {
      acc[opening].averageAccuracy = accuracy
    }
    acc[opening].results = {}
    acc[opening].results[result] = 1
    return acc
  }, {}))
}

module.exports = {
  formatGames,
  groupByOpening
}
