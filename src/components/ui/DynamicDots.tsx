import { useState, useEffect } from 'react';

interface DynamicDotsProps {
    isLoading: boolean;
}

const DynamicDots: React.FC<DynamicDotsProps> = ({ isLoading }) => {
    const [dots, setDots] = useState('');

    useEffect(() => {
        let interval: NodeJS.Timeout;
        if (isLoading) {
            interval = setInterval(() => {
                setDots((prevDots) => (prevDots.length < 3 ? prevDots + '.' : ''));
            }, 500); // Update dots every 500ms
        } else {
            setDots(''); // Reset dots when loading is stopped
        }
        return () => clearInterval(interval); // Clean up on unmount
    }, [isLoading]);

    return dots;
};

export default DynamicDots;
