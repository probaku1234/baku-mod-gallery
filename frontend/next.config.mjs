/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "ac-p1.namu.la",
        port: "",
        pathname: "/**",
      },
      {
        protocol: "https",
        hostname: "c10.patreonusercontent.com",
        port: "",
        pathname: "/**",
      }
    ],
  },
};

export default nextConfig;
