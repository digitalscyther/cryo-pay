ARG NODE_VERSION=20.17.0

FROM node:${NODE_VERSION}-alpine as base

WORKDIR /usr/src/app

FROM base as deps

RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci --omit=dev

FROM deps as build

ARG BASE_API_URL

RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci

COPY . .

RUN REACT_APP_BASE_API_URL=${BASE_API_URL} \
  npm run build

FROM base as final

ENV NODE_ENV production

# Install serve to serve the production build
RUN npm install -g serve

USER node

COPY package.json .

COPY --from=deps /usr/src/app/node_modules ./node_modules
COPY --from=build /usr/src/app/build ./build

EXPOSE 3000

# Use serve to serve the build folder
CMD ["npx", "serve", "-s", "build"]
