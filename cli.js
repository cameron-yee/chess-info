const { argv } = require('node:process')
const { fetchGamesByUsernameAndYear } = require('./fetch')
const { filterUserGamesByTimeControlAndPieces } = require('./filter')
const { formatGames, groupByOpening } = require('./format')


async function cli() {
  const [_, __, ...filters] = argv
  if (filters.length !== 4 || filters.includes('-h') || filters.includes('--help')) {
    console.error('node chess-info.js [white|black] [rapid|blitz|bullet] <username> <year>')
    return
  }
  const [pieces, timeClass, username, year] = filters

  const games = await fetchGamesByUsernameAndYear(username, year)
  const filteredGames = games.filter(filterUserGamesByTimeControlAndPieces(username, pieces, timeClass))
  const formattedGames = filteredGames.map(formatGames(username))
  const output = groupByOpening(formattedGames)
  console.log(JSON.stringify(output, null, 2))
}

cli()
