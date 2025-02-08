import clsx from 'clsx';
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faTelegram, faTwitter, faGithub, } from "@fortawesome/free-brands-svg-icons";
import { faEnvelope } from "@fortawesome/free-solid-svg-icons";
import notionIcon from "@/assets/notion-icon.png";
import PoweredByICP from '@/components/ui/PoweredByICP';
import icpLogo from "@/assets/blockchains/icp-logo.svg";

const Footer = () => {
    return (
        <footer className={clsx(
            "w-full relative py-6 px-10",
            "text-gray-700 dark:text-gray-300",
            "bg-gray-50 dark:bg-gray-900",
            "border-t border-t-gray-300 dark:border-t-gray-700"
        )}>
            <div className="flex justify-between items-center">
                <span className="text-sm">
                    &copy; 2025 - icRamp
                </span>
                <div className="flex justify-center items-center text-blue-700 dark:text-blue-300">
                    <PoweredByICP className="hidden sm:block h-4 text-gray-700 dark:text-gray-300" />
                    {/* Small screen: ICP logo */}
                    <img src={icpLogo} alt="ICP Logo" className="block sm:hidden h-4" />
                </div>
                <div className="flex space-x-4">
                    <a href="https://mesquite-structure-f75.notion.site/Onboarding-114aa21f9dd480ffb6a0ed741dddc80c" target="_blank" rel="noopener noreferrer">
                        <img src={notionIcon} alt="Notion Logo" title="Onboarding Docs" className="h-6 w-6" />
                    </a>
                    <a href="https://t.me/+1qd_xreS_hpkMTBk" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faTelegram} size="lg" color="#24A1DE" />
                    </a>
                    <a href="https://x.com/ic_rampXYZ?t=kjzM0v-CJiSfGR_RC8qSCg&s=09" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faTwitter} size="lg" color="#1DA1F2" />
                    </a>
                    <a href="https://github.com/reymom/ic2P2ramp" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faGithub} size="lg" className="black dark:text-gray-500" />
                    </a>
                    <a href="mailto:icramp.xyz@gmail.com" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faEnvelope} size="lg" color="#6f6e73" className="text-gray-800 dark:text-gray-300" />
                    </a>
                </div>
            </div>
        </footer>
    );
};

export default Footer;