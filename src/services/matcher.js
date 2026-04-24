import { getDefaultFile } from './storage';

const commonMaterialNames = [
  '身份证',
  '毕业证',
  '学位证',
  '学历证书电子注册备案表',
  '学籍在线验证报告',
  '简历',
  '英语四六级证书',
  '党员证明',
  '获奖证书',
  '资格证书',
  '证件照',
];

export function extractByRules(requirementText, materials) {
  const text = requirementText || '';
  const requiredNames = commonMaterialNames.filter((name) => text.includes(name));
  const uniqueNames = Array.from(new Set(requiredNames));

  return uniqueNames.map((requiredName) => {
    const matched = findBestMaterial(requiredName, materials);
    const file = matched ? getDefaultFile(matched) : null;

    return {
      requiredMaterial: requiredName,
      matchedMaterialId: matched ? matched.id : '',
      matchedMaterialName: matched ? matched.name : '',
      suggestedFilename: buildSuggestedFilename(requiredName, file),
      confidence: matched ? 0.86 : 0.2,
      status: matched ? 'matched' : 'missing',
      reason: matched ? '根据材料名或别名匹配' : '材料库中未找到匹配项',
    };
  });
}

export function findBestMaterial(requiredName, materials) {
  return materials.find((material) => {
    const names = [material.name, ...(material.aliases || [])];
    return names.some((name) => requiredName.includes(name) || name.includes(requiredName));
  });
}

function buildSuggestedFilename(requiredName, file) {
  const ext = file && file.originalFilename.includes('.')
    ? file.originalFilename.split('.').pop()
    : 'pdf';
  return `${requiredName}.${ext}`;
}

export function buildLlmPrompt(requirementText, materials) {
  const materialCatalog = materials.map((material) => ({
    id: material.id,
    name: material.name,
    aliases: material.aliases || [],
    defaultFile: getDefaultFile(material)
      ? getDefaultFile(material).originalFilename
      : null,
  }));

  return {
    instructions: '从招聘材料要求中提取需要提交的材料，并匹配用户材料库。只返回 JSON，不要返回 Markdown。',
    response_format: {
      type: 'json_schema',
      json_schema: {
        name: 'document_requirement_extraction',
        schema: {
          type: 'object',
          properties: {
            items: {
              type: 'array',
              items: {
                type: 'object',
                properties: {
                  required_material: { type: 'string' },
                  matched_material_id: { type: 'string' },
                  matched_material_name: { type: 'string' },
                  suggested_filename: { type: 'string' },
                  confidence: { type: 'number' },
                  reason: { type: 'string' },
                },
                required: [
                  'required_material',
                  'matched_material_id',
                  'matched_material_name',
                  'suggested_filename',
                  'confidence',
                  'reason',
                ],
              },
            },
          },
          required: ['items'],
        },
      },
    },
    material_catalog: materialCatalog,
    requirement_text: requirementText,
  };
}
