import { useState, useEffect } from 'react';
import { Badge } from '../../ui/badge';
import { Star, Settings } from 'lucide-react';
import { Button } from '../../ui/button';
import { getSystemPrompt } from '../../../api/sdk.gen';
import type { SystemPrompt } from '../../../api/types.gen';

interface SystemPromptDisplayProps {
  className?: string;
  onOpenSettings?: () => void;
}

export default function SystemPromptDisplay({
  className,
  onOpenSettings,
}: SystemPromptDisplayProps) {
  const [currentPrompt, setCurrentPrompt] = useState<SystemPrompt | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadCurrentPrompt();

    // Listen for changes to localStorage
    const handleStorageChange = (e: { key: string | null }) => {
      // StorageEvent is fired when localStorage changes from another tab
      if (e.key === 'selectedSystemPromptId') {
        loadCurrentPrompt();
      }
    };

    // Listen for custom events when localStorage is updated from the same tab
    const handlePromptChange = () => {
      loadCurrentPrompt();
    };

    window.addEventListener('storage', handleStorageChange);
    window.addEventListener('systemPromptChanged', handlePromptChange);

    return () => {
      window.removeEventListener('storage', handleStorageChange);
      window.removeEventListener('systemPromptChanged', handlePromptChange);
    };
  }, []);

  const loadCurrentPrompt = async () => {
    setLoading(true);
    try {
      // Get selected system prompt ID from localStorage
      const savedId = localStorage.getItem('selectedSystemPromptId');
      
      if (savedId && savedId !== 'undefined') {
        const response = await getSystemPrompt({ path: { id: savedId } });
        setCurrentPrompt(response.data?.prompt || null);
      } else {
        // No specific prompt selected, show default
        setCurrentPrompt(null);
      }
    } catch (error) {
      // If we can't load the selected prompt, fall back to default
      setCurrentPrompt(null);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className={className}>
        <div className="bg-background-default rounded-lg p-3 border border-borderSubtle">
          <div className="animate-pulse">
            <div className="h-4 bg-gray-200 rounded w-1/4 mb-2"></div>
            <div className="h-3 bg-gray-200 rounded w-3/4"></div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={className}>
      <div className="bg-background-default rounded-lg p-3 border border-borderSubtle">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm font-medium text-textStandard">System Prompt</span>
          {onOpenSettings && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onOpenSettings}
              className="h-6 w-6 p-0"
              title="Open Settings"
            >
              <Settings className="h-3 w-3" />
            </Button>
          )}
        </div>
        
        {currentPrompt ? (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-textStandard">
                {currentPrompt.name}
              </span>
              {currentPrompt.is_default && (
                <Badge variant="default" className="text-xs">
                  <Star className="h-3 w-3 mr-1" />
                  Default
                </Badge>
              )}
              {currentPrompt.model_specific && (
                <Badge variant="secondary" className="text-xs">
                  {currentPrompt.model_specific}
                </Badge>
              )}
            </div>
            {currentPrompt.description && (
              <p className="text-xs text-textSubtle line-clamp-2">
                {currentPrompt.description}
              </p>
            )}
            {currentPrompt.tags.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {currentPrompt.tags.map((tag, index) => (
                  <Badge key={index} variant="outline" className="text-xs">
                    {tag}
                  </Badge>
                ))}
              </div>
            )}
          </div>
        ) : (
          <div>
            <div className="flex items-center gap-2 mb-1">
              <span className="text-sm font-medium text-textStandard">Default System Prompt</span>
              <Badge variant="default" className="text-xs">
                <Star className="h-3 w-3 mr-1" />
                Default
              </Badge>
            </div>
            <p className="text-xs text-textSubtle">
              Using the built-in default system prompt for conversations
            </p>
          </div>
        )}
      </div>
    </div>
  );
}