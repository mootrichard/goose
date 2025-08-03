import { useState, useEffect } from 'react';
import { Button } from '../../ui/button';
import { Card } from '../../ui/card';
import { Badge } from '../../ui/badge';
import { Separator } from '../../ui/separator';
import { ScrollArea } from '../../ui/scroll-area';
import { Plus, Edit, Trash2, Star, StarOff, Eye } from 'lucide-react';
import SystemPromptSelector from './SystemPromptSelector';
import { 
  Dialog, 
  DialogContent, 
  DialogDescription, 
  DialogHeader, 
  DialogTitle, 
  DialogTrigger 
} from '../../ui/dialog';
import { Textarea } from '../../ui/textarea';
import { Input } from '../../ui/input';
import { Label } from '../../ui/label';
import { Switch } from '../../ui/switch';
import { 
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '../../ui/alert-dialog';
import { toastError, toastSuccess } from '../../../toasts';
import { 
  listSystemPrompts,
  createSystemPrompt, 
  updateSystemPrompt,
  deleteSystemPrompt,
  setDefaultSystemPrompt
} from '../../../api/sdk.gen';
import type { SystemPrompt as SystemPromptType } from '../../../api/types.gen';


interface CreatePromptForm {
  name: string;
  description: string;
  content: string;
  tags: string;
  model_specific: string;
  is_default: boolean;
}

interface EditPromptForm extends CreatePromptForm {
  id: string;
}

export default function SystemPromptsSection() {
  const [prompts, setPrompts] = useState<SystemPromptType[]>([]);
  const [loading, setLoading] = useState(false);
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [viewDialogOpen, setViewDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [selectedPrompt, setSelectedPrompt] = useState<SystemPromptType | null>(null);
  const [selectedSystemPromptId, setSelectedSystemPromptId] = useState<string | undefined>();
  const [createForm, setCreateForm] = useState<CreatePromptForm>({
    name: '',
    description: '',
    content: '',
    tags: '',
    model_specific: '',
    is_default: false
  });
  const [editForm, setEditForm] = useState<EditPromptForm>({
    id: '',
    name: '',
    description: '',
    content: '',
    tags: '',
    model_specific: '',
    is_default: false
  });


  useEffect(() => {
    loadPrompts();
    // Load selected system prompt from localStorage
    const saved = localStorage.getItem('selectedSystemPromptId');
    if (saved && saved !== 'undefined') {
      setSelectedSystemPromptId(saved);
    }
  }, []);

  // Save selected system prompt to localStorage whenever it changes
  useEffect(() => {
    if (selectedSystemPromptId) {
      localStorage.setItem('selectedSystemPromptId', selectedSystemPromptId);
    } else {
      localStorage.removeItem('selectedSystemPromptId');
    }
    // Dispatch custom event to notify other components
    window.dispatchEvent(new CustomEvent('systemPromptChanged'));
  }, [selectedSystemPromptId]);

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

  const handleCreatePrompt = async () => {
    try {
      const tags = createForm.tags.split(',').map(tag => tag.trim()).filter(tag => tag);
      
      await createSystemPrompt({
        body: {
          name: createForm.name,
          description: createForm.description || undefined,
          content: createForm.content,
          tags: tags.length > 0 ? tags : undefined,
          model_specific: createForm.model_specific || undefined,
          is_default: createForm.is_default || undefined,
        }
      });

      toastSuccess({
        title: "Success",
        msg: "System prompt created successfully",
      });

      setCreateDialogOpen(false);
      setCreateForm({
        name: '',
        description: '',
        content: '',
        tags: '',
        model_specific: '',
        is_default: false
      });
      loadPrompts();
    } catch (error) {
      toastError({
        title: "Error",
        msg: "Failed to create system prompt",
      });
    }
  };

  const handleEditPrompt = async () => {
    if (!editForm.id) return;

    try {
      const tags = editForm.tags.split(',').map(tag => tag.trim()).filter(tag => tag);
      
      await updateSystemPrompt({
        path: { id: editForm.id },
        body: {
          name: editForm.name || undefined,
          description: editForm.description || undefined,
          content: editForm.content || undefined,
          tags: tags.length > 0 ? tags : undefined,
          model_specific: editForm.model_specific || undefined,
        }
      });

      toastSuccess({
        title: "Success",
        msg: "System prompt updated successfully",
      });

      setEditDialogOpen(false);
      loadPrompts();
    } catch (error) {
      toastError({
        title: "Error",
        msg: "Failed to update system prompt",
      });
    }
  };

  const handleDeletePrompt = async () => {
    if (!selectedPrompt) return;

    try {
      await deleteSystemPrompt({
        path: { id: selectedPrompt.id }
      });
      
      toastSuccess({
        title: "Success",
        msg: "System prompt deleted successfully",
      });

      setDeleteDialogOpen(false);
      setSelectedPrompt(null);
      loadPrompts();
    } catch (error) {
      toastError({
        title: "Error",
        msg: (error as Error)?.message || "Failed to delete system prompt",
      });
    }
  };

  const handleSetDefault = async (prompt: SystemPromptType) => {
    try {
      await setDefaultSystemPrompt({
        path: { id: prompt.id }
      });
      
      toastSuccess({
        title: "Success",
        msg: `Set "${prompt.name}" as default system prompt`,
      });

      loadPrompts();
    } catch (error) {
      toastError({
        title: "Error",
        msg: "Failed to set default system prompt",
      });
    }
  };

  const openEditDialog = (prompt: SystemPromptType) => {
    setEditForm({
      id: prompt.id,
      name: prompt.name,
      description: prompt.description || '',
      content: prompt.content,
      tags: prompt.tags.join(', '),
      model_specific: prompt.model_specific || '',
      is_default: prompt.is_default
    });
    setEditDialogOpen(true);
  };

  const openViewDialog = (prompt: SystemPromptType) => {
    setSelectedPrompt(prompt);
    setViewDialogOpen(true);
  };

  const openDeleteDialog = (prompt: SystemPromptType) => {
    setSelectedPrompt(prompt);
    setDeleteDialogOpen(true);
  };

  return (
    <div className="space-y-6">
      {/* Default System Prompt Selection */}
      <div className="space-y-3">
        <div>
          <h3 className="text-lg font-medium">Default System Prompt</h3>
          <p className="text-sm text-muted-foreground">
            Choose the system prompt to use for new conversations
          </p>
        </div>
        <SystemPromptSelector
          value={selectedSystemPromptId}
          onChange={setSelectedSystemPromptId}
          placeholder="Use default system prompt"
          allowClear={true}
        />
      </div>

      <Separator />

      {/* System Prompt Management */}
      <div className="flex justify-between items-center">
        <div>
          <h3 className="text-lg font-medium">Manage System Prompts</h3>
          <p className="text-sm text-muted-foreground">
            Create, edit, and organize your custom system prompts
          </p>
        </div>
        <Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Create Prompt
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>Create System Prompt</DialogTitle>
              <DialogDescription>
                Create a new system prompt that can be used in sessions and recipes.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="name">Name *</Label>
                  <Input
                    id="name"
                    value={createForm.name}
                    onChange={(e) => setCreateForm(prev => ({ ...prev, name: e.target.value }))}
                    placeholder="e.g., Code Reviewer"
                  />
                </div>
                <div>
                  <Label htmlFor="model">Model (Optional)</Label>
                  <Input
                    id="model"
                    value={createForm.model_specific}
                    onChange={(e) => setCreateForm(prev => ({ ...prev, model_specific: e.target.value }))}
                    placeholder="e.g., gpt-4, claude-3"
                  />
                </div>
              </div>
              
              <div>
                <Label htmlFor="description">Description</Label>
                <Input
                  id="description"
                  value={createForm.description}
                  onChange={(e) => setCreateForm(prev => ({ ...prev, description: e.target.value }))}
                  placeholder="Brief description of this prompt's purpose"
                />
              </div>

              <div>
                <Label htmlFor="tags">Tags</Label>
                <Input
                  id="tags"
                  value={createForm.tags}
                  onChange={(e) => setCreateForm(prev => ({ ...prev, tags: e.target.value }))}
                  placeholder="coding, review, debug (comma-separated)"
                />
              </div>

              <div>
                <Label htmlFor="content">Content *</Label>
                <Textarea
                  id="content"
                  value={createForm.content}
                  onChange={(e) => setCreateForm(prev => ({ ...prev, content: e.target.value }))}
                  placeholder="Enter the system prompt content..."
                  className="min-h-[200px]"
                />
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="is_default"
                  checked={createForm.is_default}
                  onCheckedChange={(checked) => setCreateForm(prev => ({ ...prev, is_default: checked }))}
                />
                <Label htmlFor="is_default">Set as default system prompt</Label>
              </div>

              <div className="flex justify-end space-x-2">
                <Button variant="outline" onClick={() => setCreateDialogOpen(false)}>
                  Cancel
                </Button>
                <Button 
                  onClick={handleCreatePrompt}
                  disabled={!createForm.name || !createForm.content}
                >
                  Create Prompt
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      <Separator />

      {loading ? (
        <div className="flex items-center justify-center h-32">
          <div className="text-sm text-muted-foreground">Loading system prompts...</div>
        </div>
      ) : (
        <ScrollArea className="h-96">
          <div className="space-y-3">
            {prompts.map((prompt) => (
              <Card key={prompt.id} className="p-4">
                <div className="flex justify-between items-start">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-2">
                      <h4 className="font-medium truncate">{prompt.name}</h4>
                      {prompt.is_default && (
                        <Badge variant="default" className="text-xs">
                          <Star className="h-3 w-3 mr-1" />
                          Default
                        </Badge>
                      )}
                      {prompt.model_specific && (
                        <Badge variant="secondary" className="text-xs">
                          {prompt.model_specific}
                        </Badge>
                      )}
                    </div>
                    {prompt.description && (
                      <p className="text-sm text-muted-foreground mb-2 line-clamp-2">
                        {prompt.description}
                      </p>
                    )}
                    {prompt.tags.length > 0 && (
                      <div className="flex flex-wrap gap-1 mb-2">
                        {prompt.tags.map((tag, index) => (
                          <Badge key={index} variant="outline" className="text-xs">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    )}
                    <p className="text-xs text-muted-foreground">
                      Updated {new Date(prompt.updated_at).toLocaleDateString()}
                    </p>
                  </div>
                  
                  <div className="flex items-center space-x-1 ml-4">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => openViewDialog(prompt)}
                      title="View content"
                    >
                      <Eye className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => openEditDialog(prompt)}
                      title="Edit prompt"
                    >
                      <Edit className="h-4 w-4" />
                    </Button>
                    {!prompt.is_default && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleSetDefault(prompt)}
                        title="Set as default"
                      >
                        <StarOff className="h-4 w-4" />
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => openDeleteDialog(prompt)}
                      title="Delete prompt"
                      disabled={prompt.is_default}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </Card>
            ))}

            {prompts.length === 0 && (
              <div className="text-center py-8">
                <p className="text-muted-foreground">No system prompts found.</p>
                <p className="text-sm text-muted-foreground mt-1">
                  Create your first custom system prompt to get started.
                </p>
              </div>
            )}
          </div>
        </ScrollArea>
      )}

      {/* View Dialog */}
      <Dialog open={viewDialogOpen} onOpenChange={setViewDialogOpen}>
        <DialogContent className="max-w-4xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>{selectedPrompt?.name}</DialogTitle>
            <DialogDescription>
              {selectedPrompt?.description || "System prompt content"}
            </DialogDescription>
          </DialogHeader>
          <ScrollArea className="h-96">
            <div className="space-y-4">
              {selectedPrompt && (
                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <strong>ID:</strong> {selectedPrompt.id.substring(0, 8)}...
                    </div>
                    <div>
                      <strong>Default:</strong> {selectedPrompt.is_default ? 'Yes' : 'No'}
                    </div>
                    <div>
                      <strong>Model:</strong> {selectedPrompt.model_specific || 'Any'}
                    </div>
                    <div>
                      <strong>Tags:</strong> {selectedPrompt.tags.join(', ') || 'None'}
                    </div>
                  </div>
                  <Separator />
                  <div>
                    <strong className="text-sm">Content:</strong>
                    <div className="mt-2 p-3 bg-muted rounded-md text-sm whitespace-pre-wrap">
                      {selectedPrompt.content}
                    </div>
                  </div>
                </div>
              )}
            </div>
          </ScrollArea>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Edit System Prompt</DialogTitle>
            <DialogDescription>
              Update the system prompt details and content.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="edit-name">Name</Label>
                <Input
                  id="edit-name"
                  value={editForm.name}
                  onChange={(e) => setEditForm(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="e.g., Code Reviewer"
                />
              </div>
              <div>
                <Label htmlFor="edit-model">Model</Label>
                <Input
                  id="edit-model"
                  value={editForm.model_specific}
                  onChange={(e) => setEditForm(prev => ({ ...prev, model_specific: e.target.value }))}
                  placeholder="e.g., gpt-4, claude-3"
                />
              </div>
            </div>
            
            <div>
              <Label htmlFor="edit-description">Description</Label>
              <Input
                id="edit-description"
                value={editForm.description}
                onChange={(e) => setEditForm(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Brief description of this prompt's purpose"
              />
            </div>

            <div>
              <Label htmlFor="edit-tags">Tags</Label>
              <Input
                id="edit-tags"
                value={editForm.tags}
                onChange={(e) => setEditForm(prev => ({ ...prev, tags: e.target.value }))}
                placeholder="coding, review, debug (comma-separated)"
              />
            </div>

            <div>
              <Label htmlFor="edit-content">Content</Label>
              <Textarea
                id="edit-content"
                value={editForm.content}
                onChange={(e) => setEditForm(prev => ({ ...prev, content: e.target.value }))}
                placeholder="Enter the system prompt content..."
                className="min-h-[200px]"
              />
            </div>

            <div className="flex justify-end space-x-2">
              <Button variant="outline" onClick={() => setEditDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleEditPrompt}>
                Update Prompt
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Delete Dialog */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete System Prompt</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{selectedPrompt?.name}"? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeletePrompt} className="bg-destructive text-destructive-foreground">
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}