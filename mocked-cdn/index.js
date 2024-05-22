const express = require('express')
const fs = require('fs')
const path = require('path')

const app = express()
const PORT = 3000

app.get('/client-name/client-version/:documentId', (req, res) => {
  const documentId = req.params.documentId
  const filePath = path.join(__dirname, 'documents', `${documentId}.graphql`)

  fs.readFile(filePath, 'utf8', (err, data) => {
    if (err) {
      return res.status(404).send('Document not found')
    }
    res.send(data)
  })
})

app.listen(PORT, () => {
  console.log(`Local CDN running at http://localhost:${PORT}`)
})
