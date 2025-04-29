FROM node:20 as builder

WORKDIR /app

# Copy package files and dependencies
COPY package.json package-lock.json* tsconfig.json ./
COPY public ./public
COPY src ./src
COPY index.html vite.config.ts ./

# Install dependencies
RUN npm ci

# Build the application
RUN npm run build

# Production stage with nginx
FROM nginx:alpine

# Copy the built files from the build stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration if needed
# COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Start Nginx
CMD ["nginx", "-g", "daemon off;"]