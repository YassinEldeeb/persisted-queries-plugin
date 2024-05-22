const { createServer } = require('@graphql-yoga/node')
const { buildSubgraphSchema } = require('@apollo/subgraph')
const { gql } = require('graphql-tag')

const typeDefs = gql`
  extend type Query {
    topProducts(first: Int = 5): [Product]
  }

  type Product @key(fields: "id") {
    id: ID!
    name: String
    price: Int
  }
`

const resolvers = {
  Query: {
    topProducts: (_, { first }) => {
      return Array.from({ length: first }, (_, i) => ({
        id: `product-${i + 1}`,
        name: `Product ${i + 1}`,
        price: (i + 1) * 100,
      }))
    },
  },
  Product: {
    __resolveReference: (product) => ({
      id: product.id,
      name: `Product ${product.id.split('-')[1]}`,
      price: parseInt(product.id.split('-')[1]) * 100,
    }),
  },
}

const schema = buildSubgraphSchema({ typeDefs, resolvers })

const server = createServer({
  schema,
  port: 4002,
})

server.start(() => {
  console.log('Products subgraph running at http://0.0.0.0:4002/graphql')
})
