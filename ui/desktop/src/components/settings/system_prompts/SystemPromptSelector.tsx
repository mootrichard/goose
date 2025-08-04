import { useState, useEffect } from 'react';
import { Button } from '../../ui/button';
import { Badge } from '../../ui/badge';
import { Select } from '../../ui/Select';
import { RefreshCw, Star } from 'lucide-react';
import { listSystemPrompts } from '../../../api/sdk.gen';
import type { SystemPrompt } from '../../../api/types.gen';
import { toastError } from '../../../toasts';

interface SelectOption {
  value: string;
  label: string;
  prompt?: SystemPrompt;
}

interface SystemPromptSelectorProps {
  value?: string;
  onChange?: (promptId: string | undefined) => void;
  placeholder?: string;
  className?: string;
  allowClear?: boolean;
}

export default function SystemPromptSelector({
  value,
  onChange,
  placeholder = "Select system prompt",
  className,
  allowClear = true,
}: SystemPromptSelectorProps) {
  const [prompts, setPrompts] = useState<SystemPrompt[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadPrompts();
  }, []);

  const loadPrompts = async () => {
    setLoading(true);
    try {
      const response = await listSystemPrompts();
      setPrompts(response.data?.prompts || []);
    } catch (error) {
      toastError({
        title: "Error",
        msg: "Failed to load system prompts",
      });
    } finally {
      setLoading(false);
    }
  };

  const handleValueChange = (selectedOption: unknown) => {
    const option = selectedOption as SelectOption | null;
    if (!option || option.value === 'none') {
      onChange?.(undefined);
    } else {
      onChange?.(option.value);
    }
  };

  const selectedPrompt = prompts.find(p => p.id === value);

  const options = [
    ...(allowClear ? [{ value: 'none', label: 'Use default prompt' }] : []),
    ...prompts.map(prompt => ({
      value: prompt.id,
      label: prompt.name,
      prompt: prompt,
    }))
  ];

  const selectedOption = options.find(opt => opt.value === (value || 'none'));

  return (
    <div className={className}>
      <div className="flex items-center gap-2">
        <Select 
          value={selectedOption}
          onChange={handleValueChange}
          options={options}
          isDisabled={loading}
          placeholder={loading ? "Loading..." : placeholder}
          className="flex-1"
          formatOptionLabel={(option: unknown) => {
            const typedOption = option as SelectOption;
            return (
              <div className="flex items-center gap-2 w-full">
                <span className="flex-1">{typedOption.label}</span>
                {typedOption.prompt && (
                  <div className="flex items-center gap-1">
                    {typedOption.prompt.is_default && (
                      <Star className="h-3 w-3 text-yellow-500" />
                    )}
                    {typedOption.prompt.model_specific && (
                      <Badge variant="secondary" className="text-xs">
                        {typedOption.prompt.model_specific}
                      </Badge>
                    )}
                  </div>
                )}
              </div>
            );
          }}
        />
        
        <Button
          variant="outline"
          size="sm"
          onClick={loadPrompts}
          disabled={loading}
          title="Refresh prompts"
        >
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
        </Button>
      </div>
      
      {selectedPrompt && (
        <div className="mt-2 p-2 bg-muted rounded-md text-sm">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-medium">{selectedPrompt.name}</span>
            {selectedPrompt.is_default && (
              <Badge variant="default" className="text-xs">
                <Star className="h-3 w-3 mr-1" />
                Default
              </Badge>
            )}
          </div>
          {selectedPrompt.description && (
            <p className="text-muted-foreground text-xs line-clamp-2">
              {selectedPrompt.description}
            </p>
          )}
          {selectedPrompt.tags.length > 0 && (
            <div className="flex flex-wrap gap-1 mt-1">
              {selectedPrompt.tags.map((tag, index) => (
                <Badge key={index} variant="outline" className="text-xs">
                  {tag}
                </Badge>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}