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
ARG PROJECT_NAME
ARG CONTACTS
ARG SEND_MESSAGE_URL
ARG PROJECT_GITHUB_URL

RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci

COPY . .

RUN REACT_APP_BASE_API_URL=${BASE_API_URL} \
    REACT_APP_PROJECT_NAME=${PROJECT_NAME} \
    REACT_APP_CONTACTS=${CONTACTS} \
    REACT_APP_SEND_MESSAGE_URL=${SEND_MESSAGE_URL} \
    REACT_APP_PROJECT_GITHUB_URL=${PROJECT_GITHUB_URL} \
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
