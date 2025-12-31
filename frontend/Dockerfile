# use the official Bun image
FROM oven/bun:1 AS base
WORKDIR /usr/src/app

# install dependencies into temp directory
# this will cache them and speed up future builds
FROM base AS install
RUN mkdir -p /temp/dev
COPY package.json bun.lock /temp/dev/
RUN cd /temp/dev && bun install --frozen-lockfile

# install with --production (exclude devDependencies)
RUN mkdir -p /temp/prod
COPY package.json bun.lock /temp/prod/
RUN cd /temp/prod && bun install --frozen-lockfile --production

# copy node_modules from temp directory
# then copy all project files into the image
FROM base AS prerelease
ARG VITE_API_URL
ARG VITE_ADMIN_ENCRYPTION_PASSWORD
ENV VITE_API_URL=${VITE_API_URL}
ENV VITE_ADMIN_ENCRYPTION_PASSWORD=${VITE_ADMIN_ENCRYPTION_PASSWORD}
COPY --from=install /temp/dev/node_modules node_modules
COPY . .

# build the SvelteKit app
ENV NODE_ENV=production
RUN bun run prepare
RUN bun run build

# copy production dependencies and build output into final image
FROM base AS release
COPY --from=install /temp/prod/node_modules node_modules
COPY --from=prerelease /usr/src/app/build build
COPY --from=prerelease /usr/src/app/package.json .

# run the app
USER bun
EXPOSE 5173/tcp
ENTRYPOINT [ "bun", "run", "build/index.js" ]
