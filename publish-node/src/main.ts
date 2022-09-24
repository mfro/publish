import express from 'express';
import { unlink, writeFile } from 'fs/promises';

const app = express();
const port = parseInt(process.argv[2]);

let index = 0;

function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function reply(res: express.Response, code: number, value: string | Buffer) {
  if (typeof value == 'string') {
    value = Buffer.from(value, 'utf8');
  }

  res.statusCode = code;
  res.write(value);
  res.end();
}

app.use(express.raw({
  type: _ => true,
}));

app.post('/', async (req, res) => {
  if (!(req.body instanceof Buffer)) {
    return reply(res, 400, 'invalid request\n');
  }

  let id = index++;

  const path = `data/${id}`;
  await writeFile(path, req.body);

  reply(res, 200, `${id}\n`);

  await delay(24 * 60 * 60 * 1000);
  await unlink(path);
});

app.listen(port);
