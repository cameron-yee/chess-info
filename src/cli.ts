import { argv } from 'node:process'
import { fetchGamesByUsernameAndYear } from './fetch'
import { filterUserGamesByTimeControlAndPieces } from './filter'
import { formatGames, groupByOpening } from './format'

import type { TimeClass } from './types'

async function cli(): Promise<void> {
  const [_, __, ...filters] = argv
  if (filters.length !== 4 || filters.includes('-h') || filters.includes('--help')) {
    console.error('node chess-info.js [white|black] [rapid|blitz|bullet] <username> <year>')
    return
  }
  const [pieces, timeClass, username, year] = filters

  const games = await fetchGamesByUsernameAndYear(username, year)
  const filteredGames = games.filter(filterUserGamesByTimeControlAndPieces(
      username,
      pieces as 'black' | 'white',
      timeClass as TimeClass
  ))
  const formattedGames = filteredGames.map(formatGames(username))
  const output = groupByOpening(formattedGames)
  console.log(JSON.stringify(output, null, 2))
}

cli()
