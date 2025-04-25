export interface IAccuracies {
  black: number
  white: number
}

export type TimeClass = 'bullet' | 'rapid' | 'blitz'

export type GameResult =
  | ''
  | 'win'
  | 'checkmated'
  | 'agreed'
  | 'repetition'
  | 'timeout'
  | 'resigned'
  | 'stalemate'
  | 'lose'
  | 'insufficient'
  | '50move'
  | 'abandoned'
  | 'kingofthehill'
  | 'threecheck'
  | 'timevsinsufficient'
  | 'bughousepartnerlose'

export interface IGame {
  accuracies: IAccuracies
  pgn: string
  time_class: TimeClass
  black: { username: string, result: GameResult }
  white: { username: string, result: GameResult  }
}
export interface IFormattedGame {
  accuracy: number | undefined,
  opening: string,
  result: GameResult
}

export interface IChessAPIGamesResponse {
  games: IGame[]
}

export interface IOutputEntry {
  count: number
  averageAccuracy?: number
  results: Record<GameResult, number>
}

export type IOutputJSON = Record<string, IOutputEntry>

export type IPGN = {
  tags: Record<string, string>
  moves: string
}

