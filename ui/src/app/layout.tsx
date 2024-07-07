import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import React from 'react';
import NavBar from '@/components/NavBar';
const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
    title: "Kappa",
    description: "Self-hosted serverless platform",
};

export default function RootLayout({
    children,
}: {
    children: React.ReactNode
}) {
    return (
        <html lang="en">
            <head>
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body className={`${inter.className} bg-white text-black overflow-x-hidden`}>
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 font-mono">
                    <header className="mb-8">
                        <h1 className="text-3xl font-bold mb-4">kappa!</h1>
                        <NavBar />
                    </header>
                    <main>{children}</main>
                    <footer className="mt-8 text-sm text-gray-500">
                        © {new Date().getFullYear()} Will Kimbell. All rights reserved ( lol ).
                    </footer>
                </div>
            </body>
        </html>
    );
}


