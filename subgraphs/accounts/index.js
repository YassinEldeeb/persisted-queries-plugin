const { createServer } = require('@graphql-yoga/node')
const { buildSubgraphSchema } = require('@apollo/subgraph')
const { gql } = require('graphql-tag')

const typeDefs = gql`
  extend type Query {
    me: User
    user(id: ID!): User
  }

  type User @key(fields: "id") {
    id: ID!
    name: String
  }
`

const resolvers = {
  Query: {
    me: () => ({ id: '1', name: 'John Doe' }),
    user: (_, { id }) => ({ id, name: `User ${id}` }),
  },
  User: {
    __resolveReference: (user) => ({ id: user.id, name: `User ${user.id}` }),
  },
}

const schema = buildSubgraphSchema({ typeDefs, resolvers })

const server = createServer({
  schema,
  port: 4001,
})

server.start(() => {
  console.log('Accounts subgraph running at http://0.0.0.0:4001/graphql')
})
