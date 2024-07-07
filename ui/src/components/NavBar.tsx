'use client';
import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

interface NavItem {
    name: string;
    path: string;
}

const NavBar: React.FC = () => {
    const pathname = usePathname();

    const navItems: NavItem[] = [
        { name: 'home', path: '/' },
        { name: 'contact', path: '/contact' },
        { name: 'github', path: 'https://github.com/seal/kappa' },
    ];

    return (
        <nav className="font-mono mb-8">
            <ul className="text-xl flex flex-col sm:flex-row space-y-2 sm:space-y-0 sm:space-x-4">
                {navItems.map((item) => (
                    <li key={item.path}>
                        <Link href={item.path} className={`${pathname === item.path ? 'underline' : 'hover:underline'}`}>
                            {item.name}
                        </Link>
                    </li>
                ))}
            </ul>
        </nav>
    );
};

export default NavBar;


